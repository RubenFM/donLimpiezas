//! Privilegios de administrador, puntos de restauración y procesos en ejecución.

use crate::ps;
use std::collections::HashSet;
use windows_sys::Win32::Foundation::{CloseHandle, INVALID_HANDLE_VALUE};
use windows_sys::Win32::System::Diagnostics::ToolHelp::{
    CreateToolhelp32Snapshot, Process32FirstW, Process32NextW, PROCESSENTRY32W, TH32CS_SNAPPROCESS,
};

pub fn is_admin() -> bool {
    unsafe { windows_sys::Win32::UI::Shell::IsUserAnAdmin() != 0 }
}

/// Lanza una nueva instancia de la app pidiendo elevación (UAC).
/// Devuelve error si el usuario cancela el diálogo.
pub fn relaunch_as_admin() -> Result<(), String> {
    let exe = std::env::current_exe().map_err(|e| e.to_string())?;
    let script = format!(
        "Start-Process -FilePath {} -Verb RunAs -ErrorAction Stop",
        ps::quote(&exe.to_string_lossy())
    );
    ps::run(&script)
        .map(|_| ())
        .map_err(|_| "No se concedieron permisos de administrador.".into())
}

/// Crea un punto de restauración. Windows solo permite uno cada 24 horas por
/// defecto, en cuyo caso se devuelve el aviso en lugar de un error.
pub fn create_restore_point(description: &str) -> Result<String, String> {
    ps::run(&format!(
        "Checkpoint-Computer -Description {} -RestorePointType MODIFY_SETTINGS \
         -ErrorAction Stop -WarningAction SilentlyContinue -WarningVariable w\n\
         if ($w) {{ 'AVISO: ' + $w[0] }} else {{ 'Punto de restauración creado.' }}",
        ps::quote(description)
    ))
    .map_err(|e| format!("No se pudo crear el punto de restauración (¿está activada la Protección del sistema?): {e}"))
}

/// Nombres de ejecutable (en minúsculas) de los procesos en marcha.
pub fn running_processes() -> HashSet<String> {
    let mut names = HashSet::new();
    unsafe {
        let snapshot = CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS, 0);
        if snapshot == INVALID_HANDLE_VALUE {
            return names;
        }
        let mut entry: PROCESSENTRY32W = std::mem::zeroed();
        entry.dwSize = std::mem::size_of::<PROCESSENTRY32W>() as u32;
        let mut ok = Process32FirstW(snapshot, &mut entry) != 0;
        while ok {
            let len = entry.szExeFile.iter().position(|&c| c == 0).unwrap_or(entry.szExeFile.len());
            names.insert(String::from_utf16_lossy(&entry.szExeFile[..len]).to_lowercase());
            ok = Process32NextW(snapshot, &mut entry) != 0;
        }
        CloseHandle(snapshot);
    }
    names
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn incluye_el_proceso_actual() {
        let exe = std::env::current_exe().unwrap();
        let name = exe.file_name().unwrap().to_string_lossy().to_lowercase();
        assert!(running_processes().contains(&name), "{name}");
    }
}
