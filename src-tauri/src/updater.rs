//! Actualizaciones de programas con winget.

use crate::log;
use serde::Serialize;
use std::io;
use std::os::windows::process::CommandExt;
use std::process::{Command, Output};
use std::sync::Mutex;

const CREATE_NO_WINDOW: u32 = 0x0800_0000;
const COMMON_ARGS: [&str; 3] = ["--include-unknown", "--accept-source-agreements", "--disable-interactivity"];

/// Ids de la última lista, para validar qué se puede actualizar.
#[derive(Default)]
pub struct State {
    last: Mutex<Vec<Update>>,
}

#[derive(Serialize, Clone, PartialEq, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Update {
    name: String,
    id: String,
    version: String,
    available: String,
}

#[derive(Serialize)]
pub struct UpdateResult {
    ok: bool,
    message: Option<String>,
}

fn winget(args: &[&str]) -> Result<Output, String> {
    Command::new("winget")
        .args(args)
        .creation_flags(CREATE_NO_WINDOW)
        .output()
        .map_err(|e| match e.kind() {
            io::ErrorKind::NotFound => "winget no está disponible. Instala «Instalador de aplicación» \
                                        desde Microsoft Store."
                .to_string(),
            _ => format!("No se pudo ejecutar winget: {e}"),
        })
}

/// Líneas de la salida sin el indicador de progreso que winget dibuja con `\r`.
fn clean_lines(output: &str) -> Vec<&str> {
    output
        .split('\n')
        .map(|l| {
            let l = l.trim_end_matches('\r');
            l.rsplit('\r').next().unwrap_or(l).trim_end()
        })
        .collect()
}

/// Lee la tabla de `winget upgrade`. Las columnas empiezan donde empiezan las
/// palabras de la cabecera, así que no depende del idioma.
fn parse(output: &str) -> Vec<Update> {
    let lines = clean_lines(output);
    let Some(sep) = lines
        .iter()
        .position(|l| l.chars().count() > 10 && l.chars().all(|c| c == '-'))
        .filter(|&i| i > 0)
    else {
        return Vec::new();
    };
    let header: Vec<char> = lines[sep - 1].chars().collect();
    let starts: Vec<usize> = (0..header.len())
        .filter(|&i| header[i] != ' ' && (i == 0 || header[i - 1] == ' '))
        .collect();
    if starts.len() < 4 {
        return Vec::new();
    }

    let mut updates = Vec::new();
    for line in &lines[sep + 1..] {
        let chars: Vec<char> = line.chars().collect();
        // Fin de la tabla: línea vacía, pie ("N actualizaciones disponibles") u otra sección.
        if chars.len() <= starts[3] {
            break;
        }
        let col = |i: usize| -> String {
            let end = starts.get(i + 1).copied().unwrap_or(chars.len()).min(chars.len());
            chars[starts[i]..end].iter().collect::<String>().trim().to_string()
        };
        let (name, id, version, available) = (col(0), col(1), col(2), col(3));
        if id.is_empty() || id.contains(' ') || id.contains('…') || available.is_empty() {
            continue;
        }
        updates.push(Update { name, id, version, available });
    }
    updates
}

pub fn list(state: &State) -> Result<Vec<Update>, String> {
    let mut args = vec!["upgrade"];
    args.extend(COMMON_ARGS);
    let out = winget(&args)?;
    let updates = parse(&String::from_utf8_lossy(&out.stdout));
    *state.last.lock().unwrap() = updates.clone();
    Ok(updates)
}

/// Última línea con texto útil, para explicar un fallo.
fn last_message(output: &str) -> Option<String> {
    clean_lines(output)
        .into_iter()
        .rev()
        .map(str::trim)
        .find(|l| l.chars().any(char::is_alphanumeric))
        .map(String::from)
}

pub fn update(id: &str, state: &State) -> Result<UpdateResult, String> {
    let update = state
        .last
        .lock()
        .unwrap()
        .iter()
        .find(|u| u.id == id)
        .cloned()
        .ok_or_else(|| format!("«{id}» no está en la lista de actualizaciones. Pulsa Buscar actualizaciones."))?;

    let mut args = vec!["upgrade", "--id", id, "--exact", "--accept-package-agreements"];
    args.extend(COMMON_ARGS);
    let out = winget(&args)?;
    let ok = out.status.success();
    let message = (!ok).then(|| last_message(&String::from_utf8_lossy(&out.stdout))).flatten();

    if ok {
        state.last.lock().unwrap().retain(|u| u.id != id);
    }
    let result = if ok { "actualizado".to_string() } else { format!("ERROR {}", message.as_deref().unwrap_or("")) };
    let _ = log::append(
        "updates.log",
        &format!("{id}\t{} → {}\t{result}", update.version, update.available),
    );
    Ok(UpdateResult { ok, message })
}

#[cfg(test)]
mod tests {
    use super::*;

    const ENGLISH: &str = "   - \r   \\ \r\
Name                                        Id                    Version     Available   Source\r\n\
---------------------------------------------------------------------------------------------\r\n\
7-Zip 21.07 (x64)                           7zip.7zip             21.07       26.05       winget\r\n\
Battle.net                                  Blizzard.BattleNet    Unknown     2.0.0.4232  winget\r\n\
Python Launcher                             Python.Launcher       < 3.14.7    3.14.7      winget\r\n\
37 upgrades available.\r\n\
\r\n\
The following packages have an upgrade available, but require explicit targeting for upgrade:\r\n\
Name      Id          Version  Available  Source\r\n\
------------------------------------------------\r\n\
Discord   Discord.Discord  1.0  1.1      winget\r\n";

    const SPANISH: &str = "\
Nombre                    Id.               Versión   Disponible  Origen\n\
--------------------------------------------------------------------------\n\
Configurador FNMT         FNMT.Configurador 4.0.6     5.1.2       winget\n\
Línea con nombre largo …  Paquete.Largo…    1.0       2.0         winget\n\
2 actualizaciones disponibles.\n";

    fn update(name: &str, id: &str, version: &str, available: &str) -> Update {
        Update { name: name.into(), id: id.into(), version: version.into(), available: available.into() }
    }

    #[test]
    fn lee_la_tabla_en_ingles() {
        assert_eq!(
            parse(ENGLISH),
            vec![
                update("7-Zip 21.07 (x64)", "7zip.7zip", "21.07", "26.05"),
                update("Battle.net", "Blizzard.BattleNet", "Unknown", "2.0.0.4232"),
                update("Python Launcher", "Python.Launcher", "< 3.14.7", "3.14.7"),
            ]
        );
    }

    #[test]
    fn lee_la_tabla_en_espanol_y_descarta_ids_truncados() {
        assert_eq!(parse(SPANISH), vec![update("Configurador FNMT", "FNMT.Configurador", "4.0.6", "5.1.2")]);
    }

    #[test]
    fn sin_tabla() {
        assert!(parse("No installed package found matching input criteria.").is_empty());
        assert!(parse("").is_empty());
    }

    #[test]
    fn mensaje_de_error() {
        assert_eq!(
            last_message("Found 7-Zip\r\n  ██████ 100%\r\nInstaller failed with exit code: 1603\r\n\r\n").as_deref(),
            Some("Installer failed with exit code: 1603")
        );
    }

    #[test]
    fn rechaza_ids_desconocidos() {
        let state = State::default();
        *state.last.lock().unwrap() = vec![update("Git", "Git.Git", "1", "2")];
        assert!(super::update("Malware.Pkg", &state).is_err());
    }

    /// Ejecuta winget de verdad (solo lectura): `cargo test -- --ignored --nocapture`.
    #[test]
    #[ignore]
    fn lista_actualizaciones_reales() {
        let updates = list(&State::default()).unwrap();
        for u in &updates {
            println!("{} | {} | {} → {}", u.name, u.id, u.version, u.available);
        }
        println!("{} actualizaciones", updates.len());
        assert!(!updates.is_empty());
    }
}
