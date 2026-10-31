//! Programas que se ejecutan al iniciar sesión y su activación/desactivación.
//!
//! El estado se guarda en las claves `StartupApproved`, igual que hace el
//! Administrador de tareas: nunca se borran ni se modifican las entradas originales.

use crate::{cmdline, log, ps};
use serde::{Deserialize, Serialize};
use std::borrow::Cow;
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};
use winreg::enums::{
    HKEY_CURRENT_USER, HKEY_LOCAL_MACHINE, KEY_READ, KEY_WOW64_32KEY, KEY_WOW64_64KEY, REG_BINARY,
    REG_EXPAND_SZ, REG_SZ,
};
use winreg::types::FromRegValue;
use winreg::{RegKey, RegValue};

const RUN: &str = r"Software\Microsoft\Windows\CurrentVersion\Run";
const APPROVED: &str = r"Software\Microsoft\Windows\CurrentVersion\Explorer\StartupApproved";
const STARTUP_FOLDER: &str = r"Microsoft\Windows\Start Menu\Programs\Startup";

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
}

impl Source {
    const ALL: [Source; 5] = [
        Source::HkcuRun,
        Source::HklmRun,
        Source::HklmRun32,
        Source::UserFolder,
        Source::CommonFolder,
    ];

    fn label(self) -> &'static str {
        match self {
            Source::HkcuRun => "Registro (usuario)",
            Source::HklmRun => "Registro (equipo)",
            Source::HklmRun32 => "Registro (equipo, 32 bits)",
            Source::UserFolder => "Carpeta de Inicio (usuario)",
            Source::CommonFolder => "Carpeta de Inicio (común)",
        }
    }

    /// Las entradas del equipo afectan a todos los usuarios.
    fn admin(self) -> bool {
        !matches!(self, Source::HkcuRun | Source::UserFolder)
    }

    fn root(self) -> RegKey {
        RegKey::predef(if self.admin() { HKEY_LOCAL_MACHINE } else { HKEY_CURRENT_USER })
    }

    fn approved_key(self) -> String {
        let sub = match self {
            Source::HkcuRun | Source::HklmRun => "Run",
            Source::HklmRun32 => "Run32",
            Source::UserFolder | Source::CommonFolder => "StartupFolder",
        };
        format!(r"{APPROVED}\{sub}")
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
    for source in Source::ALL {
        let approved = source.root().open_subkey_with_flags(source.approved_key(), KEY_READ).ok();
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
        entry.publisher = exe.and_then(|p| details.companies.get(&p).cloned());
    }

    entries.sort_by_key(|e| {
        let order = Source::ALL.iter().position(|s| *s == e.source);
        (order, e.display_name.to_lowercase())
    });
    entries
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

pub fn set_enabled(source: Source, name: &str, enabled: bool, is_admin: bool) -> Result<(), String> {
    check_change(source, name, is_admin, &read_entries(source))?;

    let (key, _) = source
        .root()
        .create_subkey(source.approved_key())
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
