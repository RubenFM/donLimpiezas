//! Análisis y limpieza de archivos temporales, cachés y restos del sistema.

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

enum Target {
    /// Todo lo que hay dentro de la carpeta (la carpeta en sí se conserva).
    Contents(PathBuf),
    /// Archivos de la carpeta cuyo nombre empieza por el prefijo indicado.
    FilesWithPrefix(PathBuf, &'static str),
    RecycleBin,
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
    targets: Vec<Target>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CategoryScan {
    id: &'static str,
    name: &'static str,
    description: &'static str,
    group: &'static str,
    admin: bool,
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

/// Carpetas de caché de cada perfil de un navegador basado en Chromium.
fn chromium_caches(user_data: PathBuf) -> Vec<Target> {
    let mut targets = Vec::new();
    let Ok(profiles) = fs::read_dir(&user_data) else {
        return targets;
    };
    for profile in profiles.flatten() {
        let path = profile.path();
        if !path.is_dir() {
            continue;
        }
        for sub in ["Cache", "Code Cache", "GPUCache"] {
            let dir = path.join(sub);
            if dir.is_dir() {
                targets.push(Target::Contents(dir));
            }
        }
    }
    targets
}

fn firefox_caches(profiles_dir: PathBuf) -> Vec<Target> {
    let Ok(profiles) = fs::read_dir(profiles_dir) else {
        return Vec::new();
    };
    profiles
        .flatten()
        .map(|p| p.path().join("cache2"))
        .filter(|p| p.is_dir())
        .map(Target::Contents)
        .collect()
}

fn categories() -> Vec<Category> {
    let local = env_dir("LOCALAPPDATA", r"C:\Users\Default\AppData\Local");
    let temp = env_dir("TEMP", &local.join("Temp").to_string_lossy());
    let windir = env_dir("SystemRoot", r"C:\Windows");
    let program_data = env_dir("ProgramData", r"C:\ProgramData");

    vec![
        Category {
            id: "user_temp",
            name: "Archivos temporales del usuario",
            description: "Contenido de %TEMP% con más de 24 horas.",
            group: "Windows",
            admin: false,
            min_age_hours: 24,
            targets: vec![Target::Contents(temp)],
        },
        Category {
            id: "windows_temp",
            name: "Archivos temporales de Windows",
            description: r"Contenido de C:\Windows\Temp con más de 24 horas.",
            group: "Windows",
            admin: true,
            min_age_hours: 24,
            targets: vec![Target::Contents(windir.join("Temp"))],
        },
        Category {
            id: "recycle_bin",
            name: "Papelera de reciclaje",
            description: "Vacía la papelera de todas las unidades.",
            group: "Windows",
            admin: false,
            min_age_hours: 0,
            targets: vec![Target::RecycleBin],
        },
        Category {
            id: "thumbnails",
            name: "Caché de miniaturas",
            description: "El Explorador las regenera cuando las necesita.",
            group: "Windows",
            admin: false,
            min_age_hours: 0,
            targets: vec![Target::FilesWithPrefix(
                local.join(r"Microsoft\Windows\Explorer"),
                "thumbcache_",
            )],
        },
        Category {
            id: "shader_cache",
            name: "Caché de sombreadores DirectX",
            description: "Se regenera al abrir juegos y aplicaciones 3D.",
            group: "Windows",
            admin: false,
            min_age_hours: 0,
            targets: vec![Target::Contents(local.join("D3DSCache"))],
        },
        Category {
            id: "crash_dumps",
            name: "Volcados de errores del usuario",
            description: "Informes de errores y volcados de memoria de aplicaciones.",
            group: "Windows",
            admin: false,
            min_age_hours: 0,
            targets: vec![
                Target::Contents(local.join("CrashDumps")),
                Target::Contents(local.join(r"Microsoft\Windows\WER\ReportArchive")),
                Target::Contents(local.join(r"Microsoft\Windows\WER\ReportQueue")),
            ],
        },
        Category {
            id: "system_crash_dumps",
            name: "Volcados de errores del sistema",
            description: "Minidumps de pantallazos azules e informes de errores del sistema.",
            group: "Windows",
            admin: true,
            min_age_hours: 0,
            targets: vec![
                Target::Contents(windir.join("Minidump")),
                Target::Contents(program_data.join(r"Microsoft\Windows\WER\ReportArchive")),
                Target::Contents(program_data.join(r"Microsoft\Windows\WER\ReportQueue")),
            ],
        },
        Category {
            id: "windows_update",
            name: "Descargas de Windows Update",
            description: "Instaladores de actualizaciones ya descargados.",
            group: "Windows",
            admin: true,
            min_age_hours: 0,
            targets: vec![Target::Contents(windir.join(r"SoftwareDistribution\Download"))],
        },
        Category {
            id: "chrome_cache",
            name: "Google Chrome",
            description: "Caché de todos los perfiles. No borra historial, contraseñas ni sesiones.",
            group: "Navegadores",
            admin: false,
            min_age_hours: 0,
            targets: chromium_caches(local.join(r"Google\Chrome\User Data")),
        },
        Category {
            id: "edge_cache",
            name: "Microsoft Edge",
            description: "Caché de todos los perfiles. No borra historial, contraseñas ni sesiones.",
            group: "Navegadores",
            admin: false,
            min_age_hours: 0,
            targets: chromium_caches(local.join(r"Microsoft\Edge\User Data")),
        },
        Category {
            id: "brave_cache",
            name: "Brave",
            description: "Caché de todos los perfiles. No borra historial, contraseñas ni sesiones.",
            group: "Navegadores",
            admin: false,
            min_age_hours: 0,
            targets: chromium_caches(local.join(r"BraveSoftware\Brave-Browser\User Data")),
        },
        Category {
            id: "firefox_cache",
            name: "Mozilla Firefox",
            description: "Caché de todos los perfiles. No borra historial, contraseñas ni sesiones.",
            group: "Navegadores",
            admin: false,
            min_age_hours: 0,
            targets: firefox_caches(local.join(r"Mozilla\Firefox\Profiles")),
        },
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
