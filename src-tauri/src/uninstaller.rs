//! Inventario de programas instalados y desinstalación con su desinstalador oficial.

use crate::{cmdline, log};
use serde::Serialize;
use std::ffi::OsStr;
use std::io;
use std::os::windows::ffi::OsStrExt;
use windows_sys::Win32::Foundation::CloseHandle;
use windows_sys::Win32::System::Threading::{WaitForSingleObject, INFINITE};
use windows_sys::Win32::UI::Shell::{ShellExecuteExW, SEE_MASK_NOCLOSEPROCESS, SHELLEXECUTEINFOW};
use winreg::enums::{HKEY_CURRENT_USER, HKEY_LOCAL_MACHINE, KEY_READ, KEY_WOW64_32KEY, KEY_WOW64_64KEY};
use winreg::RegKey;

const UNINSTALL: &str = r"Software\Microsoft\Windows\CurrentVersion\Uninstall";
const SW_SHOWNORMAL: i32 = 1;
const ERROR_CANCELLED: i32 = 1223;

#[derive(Clone, Copy)]
enum View {
    Hklm64,
    Hklm32,
    Hkcu,
}

impl View {
    const ALL: [View; 3] = [View::Hklm64, View::Hklm32, View::Hkcu];

    fn prefix(self) -> &'static str {
        match self {
            View::Hklm64 => "hklm64",
            View::Hklm32 => "hklm32",
            View::Hkcu => "hkcu",
        }
    }

    fn open(self) -> io::Result<RegKey> {
        let (root, flags) = match self {
            View::Hklm64 => (HKEY_LOCAL_MACHINE, KEY_READ | KEY_WOW64_64KEY),
            View::Hklm32 => (HKEY_LOCAL_MACHINE, KEY_READ | KEY_WOW64_32KEY),
            View::Hkcu => (HKEY_CURRENT_USER, KEY_READ),
        };
        RegKey::predef(root).open_subkey_with_flags(UNINSTALL, flags)
    }
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Program {
    /// `<vista>\<subclave>`: permite volver a localizar la entrada.
    id: String,
    name: String,
    publisher: Option<String>,
    version: Option<String>,
    /// Tamaño estimado en bytes.
    size: Option<u64>,
    /// `yyyy-mm-dd`.
    install_date: Option<String>,
    #[serde(skip)]
    uninstall_string: String,
}

#[derive(Serialize)]
pub struct UninstallResult {
    removed: bool,
}

/// `20240315` → `2024-03-15`. Algunos instaladores registran fechas imposibles.
fn format_date(raw: &str) -> Option<String> {
    let raw = raw.trim();
    if raw.len() != 8 || !raw.bytes().all(|b| b.is_ascii_digit()) {
        return None;
    }
    let month: u32 = raw[4..6].parse().ok()?;
    let day: u32 = raw[6..].parse().ok()?;
    ((1..=12).contains(&month) && (1..=31).contains(&day))
        .then(|| format!("{}-{}-{}", &raw[..4], &raw[4..6], &raw[6..]))
}

/// Lee una entrada de `Uninstall`; `None` si no debe mostrarse.
fn read_program(view: View, subkey: &str, key: &RegKey) -> Option<Program> {
    let text = |name: &str| {
        key.get_value::<String, _>(name)
            .ok()
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
    };
    let name = text("DisplayName")?;
    let uninstall_string = text("UninstallString")?;
    if key.get_value::<u32, _>("SystemComponent").ok() == Some(1) || text("ParentKeyName").is_some() {
        return None;
    }
    if let Some(release) = text("ReleaseType") {
        if ["Update", "Hotfix", "Security Update"].iter().any(|t| release.eq_ignore_ascii_case(t)) {
            return None;
        }
    }
    Some(Program {
        id: format!(r"{}\{subkey}", view.prefix()),
        name,
        publisher: text("Publisher"),
        version: text("DisplayVersion"),
        size: key
            .get_value::<u32, _>("EstimatedSize")
            .ok()
            .filter(|&kb| kb > 0)
            .map(|kb| u64::from(kb) * 1024),
        install_date: text("InstallDate").and_then(|d| format_date(&d)),
        uninstall_string,
    })
}

pub fn list() -> Vec<Program> {
    let mut programs = Vec::new();
    for view in View::ALL {
        let Ok(root) = view.open() else {
            continue;
        };
        for subkey in root.enum_keys().flatten() {
            // Las subclaves heredan la vista (64/32 bits) de la clave padre.
            if let Ok(key) = root.open_subkey(&subkey) {
                programs.extend(read_program(view, &subkey, &key));
            }
        }
    }
    programs.sort_by_key(|p| p.name.to_lowercase());
    programs
}

fn find(id: &str) -> Option<Program> {
    let (prefix, subkey) = id.split_once('\\')?;
    let view = View::ALL.into_iter().find(|v| v.prefix() == prefix)?;
    let key = view.open().ok()?.open_subkey(subkey).ok()?;
    read_program(view, subkey, &key)
}

/// Ejecutable y argumentos para desinstalar. Los MSI siempre con `/X`, aunque
/// registren `/I` (que abre el modo "modificar"); el código de producto sale
/// del comando o, si no está, del nombre de la subclave.
fn uninstall_command(uninstall_string: &str, subkey: &str) -> Option<(String, String)> {
    if uninstall_string.to_ascii_lowercase().contains("msiexec") {
        let code = cmdline::msi_product_code(uninstall_string)
            .or_else(|| subkey.starts_with('{').then(|| subkey.to_string()))?;
        return Some(("msiexec.exe".into(), format!("/X{code}")));
    }
    cmdline::split(uninstall_string, cmdline::system_env)
}

fn wide(s: &str) -> Vec<u16> {
    OsStr::new(s).encode_wide().chain(Some(0)).collect()
}

/// Lanza el desinstalador como lo haría el Explorador (con UAC si lo pide) y
/// espera a que termine.
fn run_and_wait(file: &str, params: &str) -> Result<(), String> {
    let file_w = wide(file);
    let params_w = wide(params);
    let mut info: SHELLEXECUTEINFOW = unsafe { std::mem::zeroed() };
    info.cbSize = std::mem::size_of::<SHELLEXECUTEINFOW>() as u32;
    info.fMask = SEE_MASK_NOCLOSEPROCESS;
    info.lpFile = file_w.as_ptr();
    info.lpParameters = if params.is_empty() { std::ptr::null() } else { params_w.as_ptr() };
    info.nShow = SW_SHOWNORMAL;

    if unsafe { ShellExecuteExW(&mut info) } == 0 {
        let err = io::Error::last_os_error();
        return Err(if err.raw_os_error() == Some(ERROR_CANCELLED) {
            "Se canceló el permiso de administrador.".into()
        } else {
            format!("No se pudo abrir el desinstalador: {err}")
        });
    }
    if !info.hProcess.is_null() {
        unsafe {
            WaitForSingleObject(info.hProcess, INFINITE);
            CloseHandle(info.hProcess);
        }
    }
    Ok(())
}

pub fn uninstall(id: &str) -> Result<UninstallResult, String> {
    let program = find(id).ok_or("El programa ya no está instalado. Pulsa Actualizar.")?;
    let subkey = id.split_once('\\').map_or("", |(_, s)| s);
    let (file, params) = uninstall_command(&program.uninstall_string, subkey)
        .ok_or_else(|| format!("No se entiende el desinstalador de «{}».", program.name))?;

    run_and_wait(&file, &params)?;

    let removed = find(id).is_none();
    let state = if removed { "desinstalado" } else { "sigue instalado" };
    let version = program.version.as_deref().unwrap_or("-");
    // La desinstalación ya ocurrió; un fallo al escribir el log no debe ocultarlo.
    let _ = log::append("uninstall.log", &format!("{}\t{version}\t{state}", program.name));
    Ok(UninstallResult { removed })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fechas_de_instalacion() {
        assert_eq!(format_date("20240315").as_deref(), Some("2024-03-15"));
        assert_eq!(format_date(" 20240315 ").as_deref(), Some("2024-03-15"));
        assert_eq!(format_date("15/03/2024"), None);
        assert_eq!(format_date("20222629"), None);
        assert_eq!(format_date(""), None);
    }

    #[test]
    fn msi_siempre_con_x() {
        let guid = "{23170F69-40C1-2702-2408-000001000000}";
        assert_eq!(
            uninstall_command(&format!("MsiExec.exe /I{guid}"), guid),
            Some(("msiexec.exe".into(), format!("/X{guid}")))
        );
        // Sin código en el comando: se usa el de la subclave.
        assert_eq!(
            uninstall_command("MsiExec.exe", guid),
            Some(("msiexec.exe".into(), format!("/X{guid}")))
        );
        assert_eq!(uninstall_command("MsiExec.exe", "Programa"), None);
    }

    #[test]
    fn desinstalador_normal() {
        assert_eq!(
            uninstall_command(r#""C:\Program Files\7-Zip\Uninstall.exe" /S"#, "7-Zip"),
            Some((r"C:\Program Files\7-Zip\Uninstall.exe".into(), "/S".into()))
        );
    }

    #[test]
    fn rechaza_programas_no_detectados() {
        assert!(uninstall(r"hkcu\DonLimpiezas-no-existe").is_err());
        assert!(uninstall(r"otra\cosa").is_err());
        assert!(uninstall("sin-separador").is_err());
    }

    /// Lee el registro real: `cargo test -- --ignored --nocapture`.
    #[test]
    #[ignore]
    fn lista_programas_reales() {
        let programs = list();
        for p in &programs {
            println!(
                "{} | {:?} | {:?} | {:?} MB | {:?} | {}",
                p.name,
                p.publisher,
                p.version,
                p.size.map(|s| s / 1_048_576),
                p.install_date,
                p.id
            );
        }
        println!("{} programas", programs.len());
    }
}
