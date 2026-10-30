//! Ejecución de scripts de PowerShell sin ventana de consola.

use base64::Engine;
use std::os::windows::process::CommandExt;
use std::process::Command;

const CREATE_NO_WINDOW: u32 = 0x0800_0000;

/// Ejecuta `script` en Windows PowerShell y devuelve su salida estándar.
///
/// El script se pasa con `-EncodedCommand` (UTF-16LE en base64) para no tener
/// que pelearse con el escapado de comillas en la línea de comandos.
pub fn run(script: &str) -> Result<String, String> {
    let full = format!(
        "$ProgressPreference = 'SilentlyContinue'\n\
         [Console]::OutputEncoding = [System.Text.Encoding]::UTF8\n\
         {script}"
    );
    let utf16: Vec<u8> = full.encode_utf16().flat_map(u16::to_le_bytes).collect();
    let encoded = base64::engine::general_purpose::STANDARD.encode(utf16);

    let out = Command::new("powershell.exe")
        .args([
            "-NoProfile",
            "-NonInteractive",
            "-ExecutionPolicy",
            "Bypass",
            "-EncodedCommand",
            encoded.as_str(),
        ])
        .creation_flags(CREATE_NO_WINDOW)
        .output()
        .map_err(|e| format!("No se pudo iniciar PowerShell: {e}"))?;

    let stdout = String::from_utf8_lossy(&out.stdout).trim().to_string();
    if out.status.success() {
        Ok(stdout)
    } else {
        let stderr = String::from_utf8_lossy(&out.stderr).trim().to_string();
        Err(if stderr.is_empty() { stdout } else { stderr })
    }
}

/// Escapa un valor para usarlo dentro de una cadena PowerShell entre comillas simples.
pub fn quote(value: &str) -> String {
    format!("'{}'", value.replace('\'', "''"))
}
