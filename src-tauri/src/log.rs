//! Registro de cambios en `%LOCALAPPDATA%\DonLimpiezas`, para poder revisarlos o deshacerlos.

use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};

fn dir() -> PathBuf {
    std::env::var_os("LOCALAPPDATA")
        .map(PathBuf::from)
        .unwrap_or_else(std::env::temp_dir)
        .join("DonLimpiezas")
}

/// Añade `line` al final de `file`, precedida de la fecha local en ISO 8601.
pub fn append(file: &str, line: &str) -> Result<(), String> {
    append_in(&dir(), file, line)
}

fn append_in(dir: &Path, file: &str, line: &str) -> Result<(), String> {
    fs::create_dir_all(dir).map_err(|e| e.to_string())?;
    let mut f = OpenOptions::new()
        .create(true)
        .append(true)
        .open(dir.join(file))
        .map_err(|e| e.to_string())?;
    writeln!(f, "{}\t{line}", chrono::Local::now().to_rfc3339()).map_err(|e| e.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn crea_la_carpeta_y_anade_sin_sobrescribir() {
        let dir = std::env::temp_dir().join(format!("donlimpiezas-log-test-{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);

        append_in(&dir, "test.log", "primera").unwrap();
        append_in(&dir, "test.log", "segunda").unwrap();

        let content = fs::read_to_string(dir.join("test.log")).unwrap();
        let lines: Vec<&str> = content.lines().collect();
        assert_eq!(lines.len(), 2);
        assert!(lines[0].ends_with("\tprimera"));
        assert!(lines[1].ends_with("\tsegunda"));
        // La fecha va antes del tabulador y es ISO 8601.
        assert!(chrono::DateTime::parse_from_rfc3339(lines[0].split('\t').next().unwrap()).is_ok());

        fs::remove_dir_all(&dir).unwrap();
    }
}
