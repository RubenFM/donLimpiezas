//! Análisis y limpieza de archivos temporales, cachés y restos del sistema.

use crate::{sysinfo, system};
use std::os::windows::process::CommandExt;
use std::process::Command;
use serde::Serialize;
use std::fs::{self, Metadata};
use std::os::windows::fs::MetadataExt;
use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime};
use windows_sys::Win32::UI::Shell::{SHEmptyRecycleBinW, SHQueryRecycleBinW, SHQUERYRBINFO};

const FILE_ATTRIBUTE_REPARSE_POINT: u32 = 0x400;
const SHERB_NOCONFIRMATION: u32 = 0x1;
const SHERB_NOPROGRESSUI: u32 = 0x2;
const SHERB_NOSOUND: u32 = 0x4;
const CREATE_NO_WINDOW: u32 = 0x0800_0000;

const WINDOWS: &str = "Windows";
const BROWSERS: &str = "Navegadores";
const APPS: &str = "Aplicaciones";
const PRIVACY: &str = "Privacidad";

enum Target {
    /// Todo lo que hay dentro de la carpeta (la carpeta en sí se conserva).
    Contents(PathBuf),
    /// Archivos de la carpeta cuyo nombre empieza por el prefijo indicado.
    FilesWithPrefix(PathBuf, &'static str),
    /// Archivos concretos; los que no existen se ignoran.
    Files(Vec<PathBuf>),
    RecycleBin,
    /// Limpieza oficial del almacén de componentes (WinSxS) con DISM. Su tamaño
    /// no se conoce de antemano: se mide el espacio libre antes y después.
    ComponentStore,
}

struct Category {
    id: &'static str,
    name: &'static str,
    description: &'static str,
    group: &'static str,
    admin: bool,
    /// Solo se tocan archivos más antiguos que esto, para no romper
    /// instalaciones o programas que estén usando sus temporales ahora mismo.
    min_age_hours: u64,
    /// Si la interfaz la marca al analizar (cuando tiene algo que limpiar).
    default_selected: bool,
    /// Consecuencia que el usuario debe conocer antes de limpiarla.
    warning: Option<&'static str>,
    /// Ejecutables (en minúsculas) que conviene cerrar antes de limpiarla.
    processes: &'static [&'static str],
    targets: Vec<Target>,
}

impl Category {
    fn new(id: &'static str, group: &'static str, name: &'static str, description: &'static str) -> Self {
        Category {
            id,
            name,
            description,
            group,
            admin: false,
            min_age_hours: 0,
            default_selected: true,
            warning: None,
            processes: &[],
            targets: Vec::new(),
        }
    }

    fn admin(mut self) -> Self {
        self.admin = true;
        self
    }

    fn min_age_hours(mut self, hours: u64) -> Self {
        self.min_age_hours = hours;
        self
    }

    /// Desmarcada por defecto y con aviso.
    fn opt_in(mut self, warning: &'static str) -> Self {
        self.default_selected = false;
        self.warning = Some(warning);
        self
    }

    fn processes(mut self, processes: &'static [&'static str]) -> Self {
        self.processes = processes;
        self
    }

    fn target(mut self, target: Target) -> Self {
        self.targets.push(target);
        self
    }

    fn targets(mut self, targets: impl IntoIterator<Item = Target>) -> Self {
        self.targets.extend(targets);
        self
    }

    /// Contenido de varias subcarpetas de `base`.
    fn dirs(self, base: &Path, subdirs: &[&str]) -> Self {
        self.targets(subdirs.iter().map(|s| Target::Contents(base.join(s))))
    }
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CategoryScan {
    id: &'static str,
    name: &'static str,
    description: &'static str,
    group: &'static str,
    admin: bool,
    default_selected: bool,
    warning: Option<&'static str>,
    /// Algún programa asociado está abierto.
    running: bool,
    /// `false` si el tamaño solo se sabe al limpiar.
    size_known: bool,
    bytes: u64,
    files: u64,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CleanResult {
    id: String,
    freed: u64,
    deleted: u64,
    failed: u64,
}

#[derive(Default)]
struct Stats {
    bytes: u64,
    files: u64,
    failed: u64,
}

fn env_dir(var: &str, fallback: &str) -> PathBuf {
    std::env::var_os(var)
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from(fallback))
}

fn subdirs(dir: &Path) -> Vec<PathBuf> {
    let Ok(entries) = fs::read_dir(dir) else {
        return Vec::new();
    };
    entries.flatten().map(|e| e.path()).filter(|p| p.is_dir()).collect()
}

/// Carpetas de caché de cada perfil de un navegador basado en Chromium.
fn chromium_caches(user_data: &Path) -> Vec<Target> {
    subdirs(user_data)
        .iter()
        .flat_map(|profile| ["Cache", "Code Cache", "GPUCache"].map(|s| profile.join(s)))
        .filter(|dir| dir.is_dir())
        .map(Target::Contents)
        .collect()
}

/// Historial y cookies de cada perfil de un navegador basado en Chromium.
/// No incluye `Login Data` (contraseñas) ni `Bookmarks`.
fn chromium_privacy(user_data: &Path) -> Vec<Target> {
    let files = subdirs(user_data)
        .iter()
        .flat_map(|profile| {
            ["History", "History-journal", r"Network\Cookies", r"Network\Cookies-journal"]
                .map(|f| profile.join(f))
        })
        .collect();
    vec![Target::Files(files)]
}

fn firefox_caches(profiles: &Path) -> Vec<Target> {
    subdirs(profiles)
        .iter()
        .map(|p| p.join("cache2"))
        .filter(|p| p.is_dir())
        .map(Target::Contents)
        .collect()
}

/// Solo cookies: el historial de Firefox está en `places.sqlite` junto a los marcadores.
fn firefox_cookies(profiles: &Path) -> Vec<Target> {
    let files = subdirs(profiles)
        .iter()
        .flat_map(|p| ["cookies.sqlite", "cookies.sqlite-wal"].map(|f| p.join(f)))
        .collect();
    vec![Target::Files(files)]
}

fn categories() -> Vec<Category> {
    let local = env_dir("LOCALAPPDATA", r"C:\Users\Default\AppData\Local");
    let roaming = env_dir("APPDATA", r"C:\Users\Default\AppData\Roaming");
    let temp = env_dir("TEMP", &local.join("Temp").to_string_lossy());
    let windir = env_dir("SystemRoot", r"C:\Windows");
    let program_data = env_dir("ProgramData", r"C:\ProgramData");
    let explorer = local.join(r"Microsoft\Windows\Explorer");
    let chrome = local.join(r"Google\Chrome\User Data");
    let edge = local.join(r"Microsoft\Edge\User Data");
    let brave = local.join(r"BraveSoftware\Brave-Browser\User Data");
    let firefox = roaming.join(r"Mozilla\Firefox\Profiles");
    let firefox_local = local.join(r"Mozilla\Firefox\Profiles");

    const CACHE_ONLY: &str = "Caché de todos los perfiles. No borra historial, contraseñas ni sesiones.";
    const HISTORY_COOKIES: &str = "Historial de navegación y cookies de todos los perfiles. No borra contraseñas ni marcadores.";
    const LOGS_OUT: &str = "Cerrará tu sesión en las webs.";

    vec![
        // Windows
        Category::new("user_temp", WINDOWS, "Archivos temporales del usuario", "Contenido de %TEMP% con más de 24 horas.")
            .min_age_hours(24)
            .target(Target::Contents(temp)),
        Category::new("windows_temp", WINDOWS, "Archivos temporales de Windows", r"Contenido de C:\Windows\Temp con más de 24 horas.")
            .admin()
            .min_age_hours(24)
            .target(Target::Contents(windir.join("Temp"))),
        Category::new("recycle_bin", WINDOWS, "Papelera de reciclaje", "Vacía la papelera de todas las unidades.")
            .target(Target::RecycleBin),
        Category::new("thumbnails", WINDOWS, "Caché de miniaturas", "El Explorador las regenera cuando las necesita.")
            .target(Target::FilesWithPrefix(explorer.clone(), "thumbcache_")),
        Category::new("icon_cache", WINDOWS, "Caché de iconos", "El Explorador la regenera; los iconos pueden tardar un momento en aparecer.")
            .target(Target::FilesWithPrefix(explorer, "iconcache_")),
        Category::new("shader_cache", WINDOWS, "Caché de sombreadores DirectX", "Se regenera al abrir juegos y aplicaciones 3D.")
            .target(Target::Contents(local.join("D3DSCache"))),
        Category::new("prefetch", WINDOWS, "Prefetch", "Windows la regenera; los programas pueden tardar algo más en abrirse la primera vez.")
            .admin()
            .target(Target::Contents(windir.join("Prefetch"))),
        Category::new("windows_logs", WINDOWS, "Logs de mantenimiento de Windows", "Registros de CBS y DISM con más de 24 horas.")
            .admin()
            .min_age_hours(24)
            .dirs(&windir.join("Logs"), &["CBS", "DISM"]),
        Category::new("crash_dumps", WINDOWS, "Volcados de errores del usuario", "Informes de errores y volcados de memoria de aplicaciones.")
            .target(Target::Contents(local.join("CrashDumps")))
            .dirs(&local.join(r"Microsoft\Windows\WER"), &["ReportArchive", "ReportQueue"]),
        Category::new("system_crash_dumps", WINDOWS, "Volcados de errores del sistema", "Minidumps de pantallazos azules e informes de errores del sistema.")
            .admin()
            .target(Target::Contents(windir.join("Minidump")))
            .dirs(&program_data.join(r"Microsoft\Windows\WER"), &["ReportArchive", "ReportQueue"]),
        Category::new("windows_update", WINDOWS, "Descargas de Windows Update", "Instaladores de actualizaciones ya descargados.")
            .admin()
            .target(Target::Contents(windir.join(r"SoftwareDistribution\Download"))),
        Category::new(
            "windows_components",
            WINDOWS,
            "Componentes antiguos de Windows Update",
            "Versiones antiguas de componentes que ya no se usan, con la limpieza oficial de Windows (DISM).",
        )
        .admin()
        .opt_in("Tarda varios minutos. Se podrán seguir desinstalando las actualizaciones.")
        .target(Target::ComponentStore),
        // Navegadores
        Category::new("chrome_cache", BROWSERS, "Google Chrome", CACHE_ONLY)
            .processes(&["chrome.exe"])
            .targets(chromium_caches(&chrome)),
        Category::new("edge_cache", BROWSERS, "Microsoft Edge", CACHE_ONLY)
            .processes(&["msedge.exe"])
            .targets(chromium_caches(&edge)),
        Category::new("brave_cache", BROWSERS, "Brave", CACHE_ONLY)
            .processes(&["brave.exe"])
            .targets(chromium_caches(&brave)),
        Category::new("firefox_cache", BROWSERS, "Mozilla Firefox", CACHE_ONLY)
            .processes(&["firefox.exe"])
            .targets(firefox_caches(&firefox_local)),
        // Aplicaciones
        Category::new("discord_cache", APPS, "Discord", "Caché de imágenes y del navegador integrado.")
            .processes(&["discord.exe"])
            .dirs(&roaming.join("discord"), &["Cache", "Code Cache", "GPUCache"]),
        Category::new("spotify_cache", APPS, "Spotify", "Canciones y datos en caché; se vuelven a descargar al reproducir.")
            .processes(&["spotify.exe"])
            .target(Target::Contents(local.join(r"Spotify\Storage"))),
        Category::new("vscode_cache", APPS, "Visual Studio Code", "Caché del editor. No borra extensiones, ajustes ni proyectos.")
            .processes(&["code.exe"])
            .dirs(&roaming.join("Code"), &["Cache", "CachedData", "Code Cache", "GPUCache"]),
        Category::new("steam_cache", APPS, "Steam", "Caché del navegador integrado de la tienda y la comunidad.")
            .processes(&["steam.exe", "steamwebhelper.exe"])
            .target(Target::Contents(local.join(r"Steam\htmlcache"))),
        // Privacidad
        Category::new("chrome_privacy", PRIVACY, "Historial y cookies de Chrome", HISTORY_COOKIES)
            .opt_in(LOGS_OUT)
            .processes(&["chrome.exe"])
            .targets(chromium_privacy(&chrome)),
        Category::new("edge_privacy", PRIVACY, "Historial y cookies de Edge", HISTORY_COOKIES)
            .opt_in(LOGS_OUT)
            .processes(&["msedge.exe"])
            .targets(chromium_privacy(&edge)),
        Category::new("brave_privacy", PRIVACY, "Historial y cookies de Brave", HISTORY_COOKIES)
            .opt_in(LOGS_OUT)
            .processes(&["brave.exe"])
            .targets(chromium_privacy(&brave)),
        Category::new("firefox_cookies", PRIVACY, "Cookies de Firefox", "Cookies de todos los perfiles. No borra historial, contraseñas ni marcadores.")
            .opt_in(LOGS_OUT)
            .processes(&["firefox.exe"])
            .targets(firefox_cookies(&firefox)),
        Category::new("recent_docs", PRIVACY, "Documentos recientes", "Lista de archivos abiertos recientemente y accesos rápidos de la barra de tareas.")
            .opt_in("Vaciará las listas de recientes del Explorador y de la barra de tareas.")
            .target(Target::Contents(roaming.join(r"Microsoft\Windows\Recent"))),
    ]
}

fn is_reparse_point(meta: &Metadata) -> bool {
    meta.file_attributes() & FILE_ATTRIBUTE_REPARSE_POINT != 0
}

fn handle_file(path: &Path, meta: &Metadata, cutoff: Option<SystemTime>, delete: bool, stats: &mut Stats) {
    if let Some(cutoff) = cutoff {
        // Si no se puede leer la fecha, mejor no tocarlo.
        if meta.modified().map(|m| m > cutoff).unwrap_or(true) {
            return;
        }
    }
    if !delete {
        stats.bytes += meta.len();
        stats.files += 1;
        return;
    }
    let mut removed = fs::remove_file(path).is_ok();
    if !removed && meta.permissions().readonly() {
        let mut perms = meta.permissions();
        perms.set_readonly(false);
        removed = fs::set_permissions(path, perms).is_ok() && fs::remove_file(path).is_ok();
    }
    if removed {
        stats.bytes += meta.len();
        stats.files += 1;
    } else {
        stats.failed += 1;
    }
}

/// Recorre `dir` sumando (o borrando) su contenido. Nunca sigue enlaces
/// simbólicos ni uniones, para no salirse de la carpeta prevista.
fn visit_dir(dir: &Path, cutoff: Option<SystemTime>, delete: bool, stats: &mut Stats) {
    let Ok(entries) = fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        let Ok(meta) = fs::symlink_metadata(&path) else {
            continue;
        };
        if is_reparse_point(&meta) {
            continue;
        }
        if meta.is_dir() {
            visit_dir(&path, cutoff, delete, stats);
            if delete {
                // Solo se elimina si ha quedado vacía.
                let _ = fs::remove_dir(&path);
            }
        } else {
            handle_file(&path, &meta, cutoff, delete, stats);
        }
    }
}

fn recycle_bin_info() -> (u64, u64) {
    let mut info = SHQUERYRBINFO {
        cbSize: std::mem::size_of::<SHQUERYRBINFO>() as u32,
        i64Size: 0,
        i64NumItems: 0,
    };
    let hr = unsafe { SHQueryRecycleBinW(std::ptr::null(), &mut info) };
    if hr < 0 {
        return (0, 0);
    }
    (info.i64Size.max(0) as u64, info.i64NumItems.max(0) as u64)
}

fn process(category: &Category, delete: bool) -> Stats {
    let cutoff = (category.min_age_hours > 0)
        .then(|| SystemTime::now() - Duration::from_secs(category.min_age_hours * 3600));
    let mut stats = Stats::default();

    for target in &category.targets {
        match target {
            Target::Contents(dir) => visit_dir(dir, cutoff, delete, &mut stats),
            Target::FilesWithPrefix(dir, prefix) => {
                let Ok(entries) = fs::read_dir(dir) else {
                    continue;
                };
                for entry in entries.flatten() {
                    let name = entry.file_name();
                    if !name.to_string_lossy().to_lowercase().starts_with(prefix) {
                        continue;
                    }
                    let path = entry.path();
                    if let Ok(meta) = fs::symlink_metadata(&path) {
                        if meta.is_file() {
                            handle_file(&path, &meta, cutoff, delete, &mut stats);
                        }
                    }
                }
            }
            Target::Files(files) => {
                for path in files {
                    if let Ok(meta) = fs::symlink_metadata(path) {
                        if meta.is_file() {
                            handle_file(path, &meta, cutoff, delete, &mut stats);
                        }
                    }
                }
            }
            Target::ComponentStore => {
                if !delete {
                    continue;
                }
                let before = sysinfo::system_drive_free();
                let ok = Command::new("Dism.exe")
                    .args(["/Online", "/Cleanup-Image", "/StartComponentCleanup"])
                    .creation_flags(CREATE_NO_WINDOW)
                    .status()
                    .is_ok_and(|s| s.success());
                if ok {
                    let after = sysinfo::system_drive_free();
                    stats.bytes += after.zip(before).map_or(0, |(a, b)| a.saturating_sub(b));
                } else {
                    stats.failed += 1;
                }
            }
            Target::RecycleBin => {
                let (bytes, items) = recycle_bin_info();
                if delete && items > 0 {
                    let flags = SHERB_NOCONFIRMATION | SHERB_NOPROGRESSUI | SHERB_NOSOUND;
                    let hr = unsafe {
                        SHEmptyRecycleBinW(std::ptr::null_mut(), std::ptr::null(), flags)
                    };
                    if hr < 0 {
                        stats.failed += items;
                        continue;
                    }
                }
                stats.bytes += bytes;
                stats.files += items;
            }
        }
    }
    stats
}

pub fn scan() -> Vec<CategoryScan> {
    let running = system::running_processes();
    categories()
        .iter()
        .map(|c| {
            let stats = process(c, false);
            CategoryScan {
                id: c.id,
                name: c.name,
                description: c.description,
                group: c.group,
                admin: c.admin,
                default_selected: c.default_selected,
                warning: c.warning,
                running: c.processes.iter().any(|p| running.contains(*p)),
                size_known: !c.targets.iter().any(|t| matches!(t, Target::ComponentStore)),
                bytes: stats.bytes,
                files: stats.files,
            }
        })
        .collect()
}

pub fn clean(ids: &[String]) -> Vec<CleanResult> {
    categories()
        .iter()
        .filter(|c| ids.iter().any(|id| id == c.id))
        .map(|c| {
            let stats = process(c, true);
            CleanResult {
                id: c.id.to_string(),
                freed: stats.bytes,
                deleted: stats.files,
                failed: stats.failed,
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    #[test]
    fn ids_unicos() {
        let cats = categories();
        let ids: HashSet<_> = cats.iter().map(|c| c.id).collect();
        assert_eq!(ids.len(), cats.len());
    }

    #[test]
    fn solo_privacidad_y_componentes_desmarcadas_y_con_aviso() {
        for c in categories() {
            let opt_in = c.group == PRIVACY || c.id == "windows_components";
            assert_eq!(c.default_selected, !opt_in, "{}", c.id);
            assert_eq!(c.warning.is_some(), opt_in, "{}", c.id);
        }
    }

    #[test]
    fn componentes_de_windows() {
        let cats = categories();
        let unknown: Vec<&str> = cats
            .iter()
            .filter(|c| c.targets.iter().any(|t| matches!(t, Target::ComponentStore)))
            .map(|c| c.id)
            .collect();
        assert_eq!(unknown, vec!["windows_components"]);
        let c = cats.iter().find(|c| c.id == "windows_components").unwrap();
        assert!(c.admin && !c.default_selected);
        // Analizar no ejecuta DISM ni suma nada.
        let stats = process(c, false);
        assert_eq!((stats.bytes, stats.files, stats.failed), (0, 0, 0));
    }

    #[test]
    fn privacidad_no_toca_contrasenas_ni_marcadores() {
        for c in categories().iter().filter(|c| c.group == PRIVACY) {
            for t in &c.targets {
                if let Target::Files(files) = t {
                    for f in files {
                        let name = f.file_name().unwrap().to_string_lossy().to_lowercase();
                        assert!(
                            !["login data", "bookmarks", "places.sqlite", "logins.json", "key4.db"].contains(&name.as_str()),
                            "{}: {name}",
                            c.id
                        );
                    }
                }
            }
        }
    }

    /// Analiza el sistema real sin borrar nada: `cargo test -- --ignored --nocapture`.
    #[test]
    #[ignore]
    fn analiza_el_sistema_real() {
        for c in scan() {
            println!(
                "[{}] {} | {} MB | {} elementos | abierto={} | marcada={}",
                c.group,
                c.id,
                c.bytes / 1_048_576,
                c.files,
                c.running,
                c.default_selected
            );
        }
    }
}
