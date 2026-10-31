//! Programas que se ejecutan al iniciar sesión y su activación/desactivación.
//!
//! El estado se guarda en las claves `StartupApproved`, igual que hace el
//! Administrador de tareas: nunca se borran ni se modifican las entradas originales.
//! Las tareas programadas se activan o desactivan como en el Programador de tareas
//! y los servicios alternan entre inicio automático y manual (nunca deshabilitado).

use crate::{cmdline, log, ps};
use serde::{Deserialize, Serialize};
use std::borrow::Cow;
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};
use winreg::enums::{
    HKEY_CURRENT_USER, HKEY_LOCAL_MACHINE, KEY_READ, KEY_SET_VALUE, KEY_WOW64_32KEY, KEY_WOW64_64KEY,
    REG_BINARY, REG_EXPAND_SZ, REG_SZ,
};
use winreg::types::FromRegValue;
use winreg::{RegKey, RegValue};

const RUN: &str = r"Software\Microsoft\Windows\CurrentVersion\Run";
const APPROVED: &str = r"Software\Microsoft\Windows\CurrentVersion\Explorer\StartupApproved";
const STARTUP_FOLDER: &str = r"Microsoft\Windows\Start Menu\Programs\Startup";
const SERVICES: &str = r"SYSTEM\CurrentControlSet\Services";
/// Servicios de seguridad que nunca se muestran aunque estén fuera de la carpeta de Windows.
const PROTECTED_SERVICES: [&str; 5] = ["WinDefend", "MDCoreSvc", "WdNisSvc", "Sense", "SecurityHealthService"];
const SERVICE_AUTO: u32 = 2;
const SERVICE_MANUAL: u32 = 3;

/// Segundos entre 1601-01-01 (época de FILETIME) y 1970-01-01, en intervalos de 100 ns.
const FILETIME_UNIX_OFFSET: u64 = 116_444_736_000_000_000;

#[derive(Clone, Copy, PartialEq, Eq, Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Source {
    HkcuRun,
    HklmRun,
    HklmRun32,
    UserFolder,
    CommonFolder,
    ScheduledTask,
    Service,
}

impl Source {
    /// Orígenes basados en `Run` y carpetas de Inicio, con estado en `StartupApproved`.
    const CLASSIC: [Source; 5] = [
        Source::HkcuRun,
        Source::HklmRun,
        Source::HklmRun32,
        Source::UserFolder,
        Source::CommonFolder,
    ];
    const ALL: [Source; 7] = [
        Source::HkcuRun,
        Source::HklmRun,
        Source::HklmRun32,
        Source::UserFolder,
        Source::CommonFolder,
        Source::ScheduledTask,
        Source::Service,
    ];

    pub(crate) fn label(self) -> &'static str {
        match self {
            Source::HkcuRun => "Registro (usuario)",
            Source::HklmRun => "Registro (equipo)",
            Source::HklmRun32 => "Registro (equipo, 32 bits)",
            Source::UserFolder => "Carpeta de Inicio (usuario)",
            Source::CommonFolder => "Carpeta de Inicio (común)",
            Source::ScheduledTask => "Tarea programada",
            Source::Service => "Servicio",
        }
    }

    pub(crate) fn from_label(label: &str) -> Option<Source> {
        Source::ALL.into_iter().find(|s| s.label() == label)
    }

    /// Las entradas del equipo afectan a todos los usuarios.
    fn admin(self) -> bool {
        !matches!(self, Source::HkcuRun | Source::UserFolder)
    }

    fn root(self) -> RegKey {
        RegKey::predef(if self.admin() { HKEY_LOCAL_MACHINE } else { HKEY_CURRENT_USER })
    }

    fn approved_key(self) -> Option<String> {
        let sub = match self {
            Source::HkcuRun | Source::HklmRun => "Run",
            Source::HklmRun32 => "Run32",
            Source::UserFolder | Source::CommonFolder => "StartupFolder",
            Source::ScheduledTask | Source::Service => return None,
        };
        Some(format!(r"{APPROVED}\{sub}"))
    }

    fn approved(self) -> Option<RegKey> {
        self.root().open_subkey_with_flags(self.approved_key()?, KEY_READ).ok()
    }

    fn folder(self) -> Option<PathBuf> {
        let base = match self {
            Source::UserFolder => std::env::var_os("APPDATA")?,
            Source::CommonFolder => std::env::var_os("ProgramData")?,
            _ => return None,
        };
        Some(PathBuf::from(base).join(STARTUP_FOLDER))
    }
}

/// Entrada tal como aparece en su origen.
struct RawEntry {
    /// Nombre del valor del registro o del archivo (con extensión). Identifica la entrada.
    name: String,
    /// Comando del registro o ruta completa del archivo.
    command: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StartupEntry {
    source: Source,
    source_label: &'static str,
    name: String,
    display_name: String,
    command: String,
    target: Option<String>,
    publisher: Option<String>,
    enabled: bool,
    admin: bool,
}

fn read_entries(source: Source) -> Vec<RawEntry> {
    if let Some(folder) = source.folder() {
        let Ok(files) = fs::read_dir(folder) else {
            return Vec::new();
        };
        return files
            .flatten()
            .filter(|f| f.path().is_file())
            .map(|f| RawEntry {
                name: f.file_name().to_string_lossy().into_owned(),
                command: f.path().to_string_lossy().into_owned(),
            })
            .filter(|e| !e.name.eq_ignore_ascii_case("desktop.ini"))
            .collect();
    }

    let view = if source == Source::HklmRun32 { KEY_WOW64_32KEY } else { KEY_WOW64_64KEY };
    let Ok(key) = source.root().open_subkey_with_flags(RUN, KEY_READ | view) else {
        return Vec::new();
    };
    key.enum_values()
        .flatten()
        .filter(|(name, value)| !name.is_empty() && matches!(value.vtype, REG_SZ | REG_EXPAND_SZ))
        .filter_map(|(name, value)| {
            let command = String::from_reg_value(&value).ok()?;
            Some(RawEntry { name, command })
        })
        .collect()
}

/// Primer byte par (o sin dato) = activado; impar = desactivado.
fn is_enabled(approval: Option<&[u8]>) -> bool {
    approval.and_then(|b| b.first()).is_none_or(|b| b % 2 == 0)
}

fn approval_bytes(enabled: bool, now: SystemTime) -> [u8; 12] {
    let mut bytes = [0u8; 12];
    if enabled {
        bytes[0] = 0x02;
    } else {
        bytes[0] = 0x03;
        let since_unix = now.duration_since(UNIX_EPOCH).unwrap_or_default();
        let filetime = FILETIME_UNIX_OFFSET + (since_unix.as_nanos() / 100) as u64;
        bytes[4..].copy_from_slice(&filetime.to_le_bytes());
    }
    bytes
}

fn exe_of(command: &str) -> Option<String> {
    cmdline::split(command, cmdline::system_env).map(|(exe, _)| exe)
}

#[derive(Deserialize)]
struct TaskInfo {
    /// Ruta completa (`\Carpeta\Nombre`): identifica la tarea.
    path: String,
    name: String,
    enabled: bool,
    command: String,
    author: Option<String>,
}

#[derive(Deserialize)]
struct ServiceInfo {
    name: String,
    display: Option<String>,
    command: String,
    auto: bool,
}

#[derive(Deserialize, Default)]
struct SystemEntries {
    tasks: Vec<TaskInfo>,
    services: Vec<ServiceInfo>,
}

fn is_protected(service: &str) -> bool {
    PROTECTED_SERVICES.iter().any(|p| p.eq_ignore_ascii_case(service))
}

/// Elementos de Windows o de Defender: nunca se muestran ni se pueden cambiar.
fn is_system(command: &str, service: Option<&str>) -> bool {
    if service.is_some_and(is_protected) {
        return true;
    }
    let Some(exe) = exe_of(command) else {
        return true;
    };
    let exe = exe.to_lowercase();
    let windir = std::env::var("SystemRoot").unwrap_or_else(|_| r"C:\Windows".into()).to_lowercase();
    exe.starts_with(&format!("{windir}\\")) || exe.contains(r"\windows defender\")
}

fn kept_path() -> PathBuf {
    log::data_dir().join("services.json")
}

/// Servicios que la app pasó a inicio manual: se siguen listando para poder reactivarlos.
fn load_kept(path: &Path) -> Vec<String> {
    fs::read_to_string(path).ok().and_then(|s| serde_json::from_str(&s).ok()).unwrap_or_default()
}

fn save_kept(path: &Path, names: &[String]) -> Result<(), String> {
    fs::create_dir_all(path.parent().unwrap_or(Path::new("."))).map_err(|e| e.to_string())?;
    fs::write(path, serde_json::to_string_pretty(names).unwrap_or_default()).map_err(|e| e.to_string())
}

/// Tareas de inicio de sesión/arranque fuera de `\Microsoft\` y servicios automáticos
/// (o pasados a manual por la app), en una sola llamada a PowerShell.
fn fetch_system(kept: &[String]) -> SystemEntries {
    let kept = kept.iter().map(|n| ps::quote(n)).collect::<Vec<_>>().join(",");
    let script = format!(
        r#"$keep = @({kept})
$tasks = @(Get-ScheduledTask | Where-Object {{
    $_.TaskPath -notlike '\Microsoft\*' -and
    ($_.Triggers | Where-Object {{ $_.CimClass.CimClassName -in 'MSFT_TaskLogonTrigger','MSFT_TaskBootTrigger' }})
}} | ForEach-Object {{
    $a = $_.Actions | Select-Object -First 1
    [pscustomobject]@{{
        path = $_.TaskPath + $_.TaskName
        name = $_.TaskName
        enabled = ($_.State -ne 'Disabled')
        command = (@($a.Execute, $a.Arguments) | Where-Object {{ $_ }}) -join ' '
        author = $_.Author
    }}
}})
$services = @(Get-CimInstance Win32_Service | Where-Object {{
    $_.PathName -and ($_.StartMode -eq 'Auto' -or $keep -contains $_.Name)
}} | ForEach-Object {{
    [pscustomobject]@{{ name = $_.Name; display = $_.DisplayName; command = $_.PathName; auto = ($_.StartMode -eq 'Auto') }}
}})
ConvertTo-Json -Compress -Depth 3 -InputObject @{{ tasks = $tasks; services = $services }}"#
    );
    ps::run(&script)
        .ok()
        .and_then(|json| serde_json::from_str(&json).ok())
        .unwrap_or_default()
}

#[derive(Deserialize, Default)]
struct Details {
    targets: HashMap<String, String>,
    companies: HashMap<String, String>,
}

/// Destino de los accesos directos y editor de los ejecutables, en una sola
/// llamada a PowerShell. Si falla, el listado se muestra sin estos datos.
fn resolve_details(links: &[String], exes: &[String]) -> Details {
    if links.is_empty() && exes.is_empty() {
        return Details::default();
    }
    let list = |paths: &[String]| paths.iter().map(|p| ps::quote(p)).collect::<Vec<_>>().join(",");
    let script = format!(
        r#"$shell = New-Object -ComObject WScript.Shell
$targets = @{{}}
foreach ($l in @({links})) {{
    try {{ $t = $shell.CreateShortcut($l).TargetPath; if ($t) {{ $targets[$l] = $t }} }} catch {{}}
}}
$companies = @{{}}
foreach ($p in @(@({exes}) + @($targets.Values))) {{
    if ($p -and -not $companies.ContainsKey($p) -and (Test-Path -LiteralPath $p -PathType Leaf)) {{
        $c = (Get-Item -LiteralPath $p).VersionInfo.CompanyName
        if ($c) {{ $companies[$p] = $c.Trim() }}
    }}
}}
ConvertTo-Json -Compress -InputObject @{{ targets = $targets; companies = $companies }}"#,
        links = list(links),
        exes = list(exes),
    );
    ps::run(&script)
        .ok()
        .and_then(|json| serde_json::from_str(&json).ok())
        .unwrap_or_default()
}

pub fn list() -> Vec<StartupEntry> {
    let mut entries = Vec::new();
    for source in Source::CLASSIC {
        let approved = source.approved();
        for raw in read_entries(source) {
            let approval = approved.as_ref().and_then(|k| k.get_raw_value(&raw.name).ok());
            let display_name = match source.folder() {
                Some(_) => raw.name.rsplit_once('.').map_or(raw.name.as_str(), |(stem, _)| stem).to_string(),
                None => raw.name.clone(),
            };
            entries.push(StartupEntry {
                source,
                source_label: source.label(),
                enabled: is_enabled(approval.as_ref().map(|v| v.bytes.as_ref())),
                admin: source.admin(),
                display_name,
                name: raw.name,
                command: raw.command,
                target: None,
                publisher: None,
            });
        }
    }

    let system = fetch_system(&load_kept(&kept_path()));
    for task in system.tasks.into_iter().filter(|t| !is_system(&t.command, None)) {
        entries.push(StartupEntry {
            source: Source::ScheduledTask,
            source_label: Source::ScheduledTask.label(),
            name: task.path,
            display_name: task.name,
            command: task.command,
            target: None,
            publisher: task.author.filter(|a| !a.is_empty()),
            enabled: task.enabled,
            admin: true,
        });
    }
    for service in system.services.into_iter().filter(|s| !is_system(&s.command, Some(&s.name))) {
        entries.push(StartupEntry {
            source: Source::Service,
            source_label: Source::Service.label(),
            display_name: service.display.filter(|d| !d.is_empty()).unwrap_or_else(|| service.name.clone()),
            name: service.name,
            command: service.command,
            target: None,
            publisher: None,
            enabled: service.auto,
            admin: true,
        });
    }

    let is_link = |e: &StartupEntry| e.source.folder().is_some() && e.command.to_ascii_lowercase().ends_with(".lnk");
    let links: Vec<String> = entries.iter().filter(|e| is_link(e)).map(|e| e.command.clone()).collect();
    let exes: Vec<String> = entries
        .iter()
        .filter(|e| !is_link(e))
        .filter_map(|e| exe_of(&e.command))
        .collect();
    let details = resolve_details(&links, &exes);

    for entry in &mut entries {
        let exe = if is_link(entry) {
            entry.target = details.targets.get(&entry.command).cloned();
            entry.target.clone()
        } else {
            exe_of(&entry.command)
        };
        // El editor del ejecutable es más útil que el autor de una tarea, si se conoce.
        entry.publisher = exe.and_then(|p| details.companies.get(&p).cloned()).or(entry.publisher.take());
    }

    entries.sort_by_key(|e| {
        let order = Source::ALL.iter().position(|s| *s == e.source);
        (order, e.display_name.to_lowercase())
    });
    entries
}

/// Estado actual de una entrada, o `None` si ya no existe.
pub(crate) fn current_state(source: Source, name: &str) -> Option<bool> {
    match source {
        Source::Service => {
            let key = RegKey::predef(HKEY_LOCAL_MACHINE).open_subkey_with_flags(format!(r"{SERVICES}\{name}"), KEY_READ).ok()?;
            return Some(key.get_value::<u32, _>("Start").ok()? == SERVICE_AUTO);
        }
        Source::ScheduledTask => {
            let (folder, task) = split_task_path(name)?;
            let state = ps::run(&format!(
                "(Get-ScheduledTask -TaskPath {} -TaskName {} -ErrorAction Stop).State",
                ps::quote(folder),
                ps::quote(task)
            ))
            .ok()?;
            return Some(state.trim() != "Disabled");
        }
        _ => {}
    }
    if !read_entries(source).iter().any(|e| e.name == name) {
        return None;
    }
    let value = source.approved().and_then(|k| k.get_raw_value(name).ok());
    Some(is_enabled(value.as_ref().map(|v| v.bytes.as_ref())))
}

/// Comprueba que se puede cambiar la entrada `(source, name)`.
fn check_change(source: Source, name: &str, is_admin: bool, existing: &[RawEntry]) -> Result<(), String> {
    if source.admin() && !is_admin {
        return Err("Cambiar programas de inicio del equipo requiere administrador.".into());
    }
    if !existing.iter().any(|e| e.name == name) {
        return Err(format!("«{name}» ya no está en {}. Pulsa Actualizar.", source.label()));
    }
    Ok(())
}

/// `\Carpeta\Nombre` → (`\Carpeta\`, `Nombre`).
fn split_task_path(path: &str) -> Option<(&str, &str)> {
    let i = path.rfind('\\')?;
    Some((&path[..=i], &path[i + 1..])).filter(|(_, name)| !name.is_empty())
}

fn set_system_enabled(source: Source, name: &str, enabled: bool, is_admin: bool) -> Result<(), String> {
    if !is_admin {
        return Err("Cambiar tareas programadas y servicios requiere administrador.".into());
    }
    if source == Source::Service && is_protected(name) {
        return Err(format!("«{name}» es un servicio de seguridad de Windows y no se puede cambiar."));
    }
    let kept_file = kept_path();
    let mut kept = load_kept(&kept_file);
    let system = fetch_system(&kept);
    let missing = || format!("«{name}» ya no está en {}. Pulsa Actualizar.", source.label());

    if source == Source::ScheduledTask {
        system.tasks.iter().find(|t| t.path == name && !is_system(&t.command, None)).ok_or_else(missing)?;
        let (folder, task) = split_task_path(name).ok_or_else(missing)?;
        let verb = if enabled { "Enable-ScheduledTask" } else { "Disable-ScheduledTask" };
        ps::run(&format!(
            "{verb} -TaskPath {} -TaskName {} -ErrorAction Stop | Out-Null",
            ps::quote(folder),
            ps::quote(task)
        ))
        .map_err(|e| format!("No se pudo cambiar «{task}»: {e}"))?;
    } else {
        system
            .services
            .iter()
            .find(|s| s.name.eq_ignore_ascii_case(name) && !is_system(&s.command, Some(&s.name)))
            .ok_or_else(missing)?;
        // Solo se cambia `Start`; `DelayedAutostart` se conserva y con él el inicio retrasado.
        let key = RegKey::predef(HKEY_LOCAL_MACHINE)
            .open_subkey_with_flags(format!(r"{SERVICES}\{name}"), KEY_SET_VALUE)
            .map_err(|e| format!("No se pudo abrir el servicio «{name}»: {e}"))?;
        key.set_value("Start", &if enabled { SERVICE_AUTO } else { SERVICE_MANUAL })
            .map_err(|e| format!("No se pudo cambiar «{name}»: {e}"))?;
        kept.retain(|k| !k.eq_ignore_ascii_case(name));
        if !enabled {
            kept.push(name.to_string());
        }
        save_kept(&kept_file, &kept)?;
    }

    let state = if enabled { "activado" } else { "desactivado" };
    let _ = log::append("startup.log", &format!("{}\t{name}\t{state}", source.label()));
    Ok(())
}

pub fn set_enabled(source: Source, name: &str, enabled: bool, is_admin: bool) -> Result<(), String> {
    if matches!(source, Source::ScheduledTask | Source::Service) {
        return set_system_enabled(source, name, enabled, is_admin);
    }
    check_change(source, name, is_admin, &read_entries(source))?;

    let approved_key = source.approved_key().ok_or("Origen no válido.")?;
    let (key, _) = source
        .root()
        .create_subkey(approved_key)
        .map_err(|e| format!("No se pudo abrir la clave de inicio: {e}"))?;
    let value = RegValue {
        bytes: Cow::Owned(approval_bytes(enabled, SystemTime::now()).to_vec()),
        vtype: REG_BINARY,
    };
    key.set_raw_value(name, &value)
        .map_err(|e| format!("No se pudo cambiar «{name}»: {e}"))?;

    let state = if enabled { "activado" } else { "desactivado" };
    // El cambio ya está hecho; un fallo al escribir el log no debe deshacerlo.
    let _ = log::append("startup.log", &format!("{}\t{name}\t{state}", source.label()));
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    fn raw(name: &str) -> RawEntry {
        RawEntry { name: name.into(), command: String::new() }
    }

    #[test]
    fn estado_de_aprobacion() {
        assert!(is_enabled(None));
        assert!(is_enabled(Some(&[])));
        assert!(is_enabled(Some(&[0x02, 0, 0, 0])));
        assert!(is_enabled(Some(&[0x06, 0, 0, 0])));
        assert!(!is_enabled(Some(&[0x03, 0, 0, 0])));
        assert!(!is_enabled(Some(&[0x07, 0, 0, 0])));
    }

    #[test]
    fn bytes_para_activar() {
        assert_eq!(approval_bytes(true, SystemTime::now()), [2, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]);
    }

    #[test]
    fn bytes_para_desactivar_incluyen_filetime() {
        // 2020-01-01T00:00:00Z = 1577836800 s Unix = 132223104000000000 en FILETIME.
        let at = UNIX_EPOCH + Duration::from_secs(1_577_836_800);
        let bytes = approval_bytes(false, at);
        assert_eq!(bytes[..4], [3, 0, 0, 0]);
        assert_eq!(u64::from_le_bytes(bytes[4..].try_into().unwrap()), 132_223_104_000_000_000);
        assert!(!is_enabled(Some(&bytes)));
    }

    #[test]
    fn excluye_elementos_de_windows_y_defender() {
        assert!(is_system(r"C:\WINDOWS\System32\AgentActivationRuntimeStarter.exe", None));
        assert!(is_system(r"%windir%\system32\SecurityHealthSystray.exe", None));
        assert!(is_system(
            r#""C:\ProgramData\Microsoft\Windows Defender\Platform\4.18\MsMpEng.exe""#,
            Some("WinDefend")
        ));
        assert!(is_system(r"C:\Program Files\Algo\x.exe", Some("mdcoresvc")));
        assert!(is_system("", None));
        assert!(!is_system(r#""C:\Program Files (x86)\GIGABYTE\EasyTune\etinit.exe""#, None));
        assert!(!is_system(r#""C:\Program Files (x86)\Google\GoogleUpdater\updater.exe" --system"#, Some("GoogleUpdaterService")));
        // Que una carpeta se llame parecido no basta.
        assert!(!is_system(r"C:\WindowsApps2\x.exe", None));
    }

    #[test]
    fn rutas_de_tareas() {
        assert_eq!(split_task_path(r"\EasyTune"), Some((r"\", "EasyTune")));
        assert_eq!(split_task_path(r"\Agent Activation Runtime\S-1-5"), Some((r"\Agent Activation Runtime\", "S-1-5")));
        assert_eq!(split_task_path(r"\Carpeta\"), None);
        assert_eq!(split_task_path("sin-barra"), None);
    }

    #[test]
    fn tareas_y_servicios_exigen_admin_y_protegen_defender() {
        assert!(set_enabled(Source::ScheduledTask, r"\EasyTune", false, false).is_err());
        assert!(set_enabled(Source::Service, "GoogleUpdaterService", false, false).is_err());
        let err = set_enabled(Source::Service, "WinDefend", false, true).unwrap_err();
        assert!(err.contains("seguridad"), "{err}");
    }

    #[test]
    fn servicios_guardados_ida_y_vuelta() {
        let path = std::env::temp_dir().join(format!("donlimpiezas-services-{}.json", std::process::id()));
        save_kept(&path, &["Gservice".into(), "brave".into()]).unwrap();
        assert_eq!(load_kept(&path), vec!["Gservice".to_string(), "brave".to_string()]);
        fs::remove_file(&path).unwrap();
        assert!(load_kept(&path).is_empty());
    }

    #[test]
    fn rechaza_entradas_inexistentes() {
        let existing = [raw("Spotify")];
        assert!(check_change(Source::HkcuRun, "Spotify", false, &existing).is_ok());
        assert!(check_change(Source::HkcuRun, "Teams", false, &existing).is_err());
    }

    #[test]
    fn rechaza_origenes_de_equipo_sin_admin() {
        let existing = [raw("SecurityHealth")];
        assert!(check_change(Source::HklmRun, "SecurityHealth", false, &existing).is_err());
        assert!(check_change(Source::CommonFolder, "SecurityHealth", false, &existing).is_err());
        assert!(check_change(Source::HklmRun, "SecurityHealth", true, &existing).is_ok());
    }

    /// Crea un acceso directo temporal a notepad.exe y lo resuelve con PowerShell.
    #[test]
    #[ignore]
    fn resuelve_destino_y_editor_de_lnk() {
        let lnk = std::env::temp_dir().join(format!("donlimpiezas-{}.lnk", std::process::id()));
        let lnk = lnk.to_string_lossy().into_owned();
        let notepad = r"C:\Windows\System32\notepad.exe".to_string();
        ps::run(&format!(
            "$s = (New-Object -ComObject WScript.Shell).CreateShortcut({}); $s.TargetPath = {}; $s.Save()",
            ps::quote(&lnk),
            ps::quote(&notepad)
        ))
        .unwrap();

        let details = resolve_details(std::slice::from_ref(&lnk), &[]);
        let _ = fs::remove_file(&lnk);

        let target = details.targets.get(&lnk).expect("destino resuelto");
        assert!(target.eq_ignore_ascii_case(&notepad), "{target}");
        assert_eq!(details.companies.get(target).map(String::as_str), Some("Microsoft Corporation"));
    }

    /// Lee el registro real: `cargo test -- --ignored --nocapture`.
    #[test]
    #[ignore]
    fn lista_entradas_reales() {
        for e in list() {
            println!(
                "[{}] {} | {} | destino={:?} | editor={:?} | activado={}",
                e.source_label, e.display_name, e.command, e.target, e.publisher, e.enabled
            );
        }
    }
}
