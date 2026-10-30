//! Comprobación y elevación de privilegios de administrador.

use crate::ps;

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
