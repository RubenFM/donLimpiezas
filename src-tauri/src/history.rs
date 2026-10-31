//! Historial unificado a partir de los logs de cada módulo y vuelta atrás.
//!
//! Deshacer siempre se recalcula desde la línea del log y se comprueba contra el
//! estado real del equipo: la interfaz solo envía el identificador de la entrada.

use crate::startup::{self, Source};
use crate::{cloud, debloat, log, system, tweaks};
use serde::Serialize;
use std::collections::HashMap;
use std::fs;
use std::path::Path;

const MAX_ENTRIES: usize = 500;

/// Logs y módulo que los escribe.
const LOGS: [(&str, &str); 8] = [
    ("debloat.log", "bloatware-remover"),
    ("startup.log", "startup-manager"),
    ("tweaks.log", "performance-optimizer"),
    ("uninstall.log", "uninstaller"),
    ("duplicates.log", "duplicate-finder"),
    ("cloud.log", "cloud-drive-cleaner"),
    ("updates.log", "software-updater"),
    ("drivers.log", "driver-updater"),
];

struct Line {
    log: &'static str,
    module: &'static str,
    /// Número de línea en su archivo; junto con el log identifica la entrada.
    index: usize,
    time: chrono::DateTime<chrono::FixedOffset>,
    fields: Vec<String>,
}

/// Lo que se puede hacer para revertir una acción.
#[derive(Debug, PartialEq)]
enum Undo {
    Startup { source: Source, name: String, enable: bool },
    TweakRevert(String),
    TweakApply(String),
    StoreSearch(String),
    RecycleBin,
    CloudKeep(String),
}

impl Undo {
    fn label(&self) -> &'static str {
        match self {
            Undo::Startup { enable: true, .. } => "Volver a activar",
            Undo::Startup { enable: false, .. } => "Volver a desactivar",
            Undo::TweakRevert(_) => "Restaurar",
            Undo::TweakApply(_) => "Volver a aplicar",
            Undo::StoreSearch(_) => "Buscar en Microsoft Store",
            Undo::RecycleBin => "Abrir la papelera",
            Undo::CloudKeep(_) => "Volver a descargar",
        }
    }
}

struct Interpreted {
    text: String,
    detail: Option<String>,
    error: Option<String>,
    undo: Option<Undo>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Entry {
    id: String,
    /// ISO 8601.
    time: String,
    module: &'static str,
    text: String,
    detail: Option<String>,
    ok: bool,
    error: Option<String>,
    /// Texto del botón, si se puede deshacer ahora.
    action: Option<&'static str>,
}

fn read_log(dir: &Path, log: &'static str, module: &'static str) -> Vec<Line> {
    let Ok(content) = fs::read_to_string(dir.join(log)) else {
        return Vec::new();
    };
    content
        .lines()
        .enumerate()
        .filter_map(|(index, raw)| {
            let mut fields = raw.trim_end_matches('\r').split('\t').map(String::from);
            let time = chrono::DateTime::parse_from_rfc3339(&fields.next()?).ok()?;
            let fields: Vec<String> = fields.collect();
            (!fields.is_empty()).then_some(Line { log, module, index, time, fields })
        })
        .collect()
}

fn file_name(path: &str) -> String {
    Path::new(path).file_name().map_or(path.into(), |n| n.to_string_lossy().into_owned())
}

/// `OK`/`instalado`/… o `ERROR mensaje`.
fn split_error(state: &str) -> Option<String> {
    state.strip_prefix("ERROR").map(|e| e.trim().to_string())
}

/// Texto de la entrada y la acción candidata (sin comprobar aún el estado real).
fn interpret(line: &Line) -> Option<Interpreted> {
    let f = &line.fields;
    let get = |i: usize| f.get(i).map(String::as_str);
    let plain = |text: String, detail: Option<String>, error: Option<String>| Interpreted { text, detail, error, undo: None };
    Some(match line.log {
        "debloat.log" => {
            let (package, state) = (get(0)?, get(1)?);
            let name = debloat::display_name(package).unwrap_or(package);
            if state == "OK" {
                Interpreted {
                    text: format!("App quitada: {name}"),
                    detail: Some(package.into()),
                    error: None,
                    undo: Some(Undo::StoreSearch(name.into())),
                }
            } else {
                plain(format!("No se pudo quitar: {name}"), Some(package.into()), split_error(state))
            }
        }
        "startup.log" => {
            let (label, name, state) = (get(0)?, get(1)?, get(2)?);
            let source = Source::from_label(label)?;
            let enabled = state == "activado";
            Interpreted {
                text: format!("{} del inicio: {name}", if enabled { "Activado" } else { "Desactivado" }),
                detail: Some(label.into()),
                error: None,
                undo: Some(Undo::Startup { source, name: name.into(), enable: !enabled }),
            }
        }
        "tweaks.log" => {
            let (id, state) = (get(0)?, get(1)?);
            let name = tweaks::info(id).map_or(id.to_string(), |t| t.name.to_string());
            match state {
                "aplicado" => Interpreted {
                    text: format!("Ajuste aplicado: {name}"),
                    detail: None,
                    error: None,
                    undo: Some(Undo::TweakRevert(id.into())),
                },
                "restaurado" => Interpreted {
                    text: format!("Ajuste restaurado: {name}"),
                    detail: None,
                    error: None,
                    undo: Some(Undo::TweakApply(id.into())),
                },
                _ => plain(format!("Error en el ajuste: {name}"), None, split_error(state)),
            }
        }
        "uninstall.log" => {
            let (name, version, state) = (get(0)?, get(1)?, get(2)?);
            let version = (version != "-").then(|| format!(" {version}")).unwrap_or_default();
            if state == "desinstalado" {
                plain(format!("Desinstalado: {name}{version}"), None, None)
            } else {
                plain(format!("No se desinstaló: {name}{version}"), None, Some("Sigue instalado".into()))
            }
        }
        "duplicates.log" => {
            let path = get(0)?;
            Interpreted {
                text: format!("Duplicado enviado a la papelera: {}", file_name(path)),
                detail: Some(path.into()),
                error: None,
                undo: Some(Undo::RecycleBin),
            }
        }
        "cloud.log" => {
            let path = get(0)?;
            // Las líneas antiguas solo tienen la ruta: eran liberaciones.
            if get(1) == Some("descargado") {
                plain(format!("Vuelto a descargar: {}", file_name(path)), Some(path.into()), None)
            } else {
                Interpreted {
                    text: format!("Liberado de este equipo: {}", file_name(path)),
                    detail: Some(path.into()),
                    error: None,
                    undo: Some(Undo::CloudKeep(path.into())),
                }
            }
        }
        "updates.log" => {
            let (id, versions, state) = (get(0)?, get(1)?, get(2)?);
            match split_error(state) {
                None => plain(format!("Programa actualizado: {id}"), Some(versions.into()), None),
                Some(e) => plain(format!("No se pudo actualizar: {id}"), Some(versions.into()), Some(e)),
            }
        }
        "drivers.log" => {
            let (title, state) = (get(0)?, get(1)?);
            match split_error(state) {
                None => plain(format!("Controlador instalado: {title}"), None, None),
                Some(e) => plain(format!("No se pudo instalar: {title}"), None, Some(e)),
            }
        }
        _ => return None,
    })
}

/// Para el inicio solo vale deshacer la última acción sobre cada programa.
fn latest_startup(lines: &[Line]) -> HashMap<(String, String), usize> {
    lines
        .iter()
        .filter(|l| l.log == "startup.log" && l.fields.len() >= 2)
        .map(|l| ((l.fields[0].clone(), l.fields[1].to_lowercase()), l.index))
        .collect()
}

fn is_latest(line: &Line, latest: &HashMap<(String, String), usize>) -> bool {
    line.log != "startup.log"
        || latest.get(&(line.fields[0].clone(), line.fields[1].to_lowercase())) == Some(&line.index)
}

/// Comprueba contra el estado real del equipo si deshacer tiene sentido ahora.
fn still_valid(undo: &Undo) -> bool {
    match undo {
        Undo::Startup { source, name, enable } => startup::current_state(*source, name) == Some(!enable),
        Undo::TweakRevert(id) => tweaks::info(id).is_some_and(|t| t.restorable),
        Undo::TweakApply(id) => tweaks::info(id).is_some_and(|t| !t.applied),
        Undo::StoreSearch(_) | Undo::RecycleBin => true,
        Undo::CloudKeep(path) => cloud::is_freed(path),
    }
}

fn all_lines(dir: &Path) -> Vec<Line> {
    LOGS.iter().flat_map(|(log, module)| read_log(dir, log, module)).collect()
}

pub fn list() -> Vec<Entry> {
    let lines = all_lines(&log::data_dir());
    let latest = latest_startup(&lines);
    let mut entries: Vec<(chrono::DateTime<chrono::FixedOffset>, Entry)> = lines
        .iter()
        .filter_map(|line| {
            let it = interpret(line)?;
            let action = it
                .undo
                .as_ref()
                .filter(|u| is_latest(line, &latest) && still_valid(u))
                .map(Undo::label);
            Some((
                line.time,
                Entry {
                    id: format!("{}:{}", line.log, line.index),
                    time: line.time.to_rfc3339(),
                    module: line.module,
                    text: it.text,
                    detail: it.detail,
                    ok: it.error.is_none(),
                    error: it.error,
                    action,
                },
            ))
        })
        .collect();
    entries.sort_by(|a, b| b.0.cmp(&a.0));
    entries.into_iter().take(MAX_ENTRIES).map(|(_, e)| e).collect()
}

/// Codifica un texto para la parte de consulta de un URI.
fn encode_query(text: &str) -> String {
    text.bytes()
        .map(|b| match b {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' => (b as char).to_string(),
            _ => format!("%{b:02X}"),
        })
        .collect()
}

/// Deshace la entrada `<log>:<línea>` y devuelve un mensaje para el usuario.
pub fn undo(id: &str, is_admin: bool) -> Result<String, String> {
    let (log_name, index) = id
        .rsplit_once(':')
        .and_then(|(l, i)| Some((l, i.parse::<usize>().ok()?)))
        .ok_or("Entrada no válida.")?;
    let (log, module) = LOGS.iter().find(|(l, _)| *l == log_name).ok_or("Entrada no válida.")?;
    let lines = read_log(&log::data_dir(), log, module);
    let line = lines.iter().find(|l| l.index == index).ok_or("La entrada ya no está en el historial.")?;
    let undo = interpret(line).and_then(|i| i.undo).ok_or("Esta acción no se puede deshacer.")?;
    if !is_latest(line, &latest_startup(&lines)) || !still_valid(&undo) {
        return Err("Ya no se puede deshacer: el estado ha cambiado desde entonces.".into());
    }

    let tweak_result = |results: Vec<tweaks::TweakResult>| -> Result<(), String> {
        match results.into_iter().next() {
            Some(r) if r.ok() => Ok(()),
            Some(r) => Err(r.error().unwrap_or("Error desconocido").to_string()),
            None => Err("Ajuste desconocido.".into()),
        }
    };
    match undo {
        Undo::Startup { source, name, enable } => {
            startup::set_enabled(source, &name, enable, is_admin)?;
            Ok(format!("«{name}» se ha {} en el inicio.", if enable { "activado" } else { "desactivado" }))
        }
        Undo::TweakRevert(id) => {
            tweak_result(tweaks::revert(&[id], is_admin)?)?;
            Ok("Ajuste restaurado a su valor original.".into())
        }
        Undo::TweakApply(id) => {
            tweak_result(tweaks::apply(&[id], is_admin)?)?;
            Ok("Ajuste aplicado de nuevo.".into())
        }
        Undo::StoreSearch(name) => {
            system::shell_open(&format!("ms-windows-store://search/?query={}", encode_query(&name)), "open")?;
            Ok(format!("Se ha abierto Microsoft Store buscando «{name}»."))
        }
        Undo::RecycleBin => {
            system::shell_open("shell:RecycleBinFolder", "open")?;
            Ok("Se ha abierto la papelera de reciclaje.".into())
        }
        Undo::CloudKeep(path) => {
            cloud::keep_local(&path)?;
            Ok(format!("«{}» se descargará y se mantendrá en este equipo.", file_name(&path)))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const T: &str = "2026-10-31T15:15:17.939852200+01:00";

    fn lines_in(name: &str, files: &[(&str, &str)]) -> (std::path::PathBuf, Vec<Line>) {
        let dir = std::env::temp_dir().join(format!("donlimpiezas-history-{name}-{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        for (file, content) in files {
            fs::write(dir.join(file), content).unwrap();
        }
        let lines = all_lines(&dir);
        (dir, lines)
    }

    fn texts(lines: &[Line]) -> Vec<String> {
        lines.iter().filter_map(interpret).map(|i| i.text).collect()
    }

    #[test]
    fn lee_todos_los_formatos() {
        let (dir, lines) = lines_in(
            "formatos",
            &[
                // debloat.log lo escribe PowerShell con 7 decimales.
                ("debloat.log", "2026-10-31T10:00:00.1234567+01:00\tMicrosoft.BingNews\tOK\r\n"),
                ("startup.log", &format!("{T}\tRegistro (usuario)\tSpotify\tdesactivado\n")),
                ("tweaks.log", &format!("{T}\tgame_dvr\taplicado\n{T}\tcopilot\tERROR sin permiso\n")),
                ("uninstall.log", &format!("{T}\t7-Zip\t21.07\tdesinstalado\n")),
                ("duplicates.log", &format!("{T}\tC:\\Users\\ana\\foto (1).jpg\n")),
                ("cloud.log", &format!("{T}\tC:\\Users\\ana\\OneDrive\\v.mp4\n{T}\tC:\\Users\\ana\\OneDrive\\v.mp4\tdescargado\n")),
                ("updates.log", &format!("{T}\tGit.Git\t2.45.1 → 2.56.0\tactualizado\n")),
                ("drivers.log", &format!("{T}\tRealtek Audio\tERROR código 4\n")),
            ],
        );
        assert_eq!(
            texts(&lines),
            vec![
                "App quitada: Noticias",
                "Desactivado del inicio: Spotify",
                "Ajuste aplicado: Grabación en segundo plano de juegos",
                "Error en el ajuste: Copilot en Windows",
                "Desinstalado: 7-Zip 21.07",
                "Duplicado enviado a la papelera: foto (1).jpg",
                "Liberado de este equipo: v.mp4",
                "Vuelto a descargar: v.mp4",
                "Programa actualizado: Git.Git",
                "No se pudo instalar: Realtek Audio",
            ]
        );
        let errors: Vec<Option<String>> = lines.iter().filter_map(interpret).map(|i| i.error).collect();
        assert_eq!(errors[3].as_deref(), Some("sin permiso"));
        assert_eq!(errors[9].as_deref(), Some("código 4"));
        fs::remove_dir_all(dir).unwrap();
    }

    #[test]
    fn ignora_lineas_danadas() {
        let (dir, lines) = lines_in(
            "danadas",
            &[("tweaks.log", &format!("basura sin fecha\n\n{T}\n{T}\tgame_dvr\taplicado\n{T}\tsolo-id\n"))],
        );
        assert_eq!(texts(&lines), vec!["Ajuste aplicado: Grabación en segundo plano de juegos"]);
        fs::remove_dir_all(dir).unwrap();
    }

    #[test]
    fn inicio_solo_deshace_la_ultima_accion() {
        let (dir, lines) = lines_in(
            "inicio",
            &[(
                "startup.log",
                &format!(
                    "{T}\tRegistro (usuario)\tSpotify\tdesactivado\n\
                     {T}\tRegistro (usuario)\tSpotify\tactivado\n\
                     {T}\tRegistro (usuario)\tDiscord\tdesactivado\n"
                ),
            )],
        );
        let latest = latest_startup(&lines);
        let latest_flags: Vec<bool> = lines.iter().map(|l| is_latest(l, &latest)).collect();
        assert_eq!(latest_flags, vec![false, true, true]);
        assert_eq!(
            interpret(&lines[1]).unwrap().undo,
            Some(Undo::Startup { source: Source::HkcuRun, name: "Spotify".into(), enable: false })
        );
        fs::remove_dir_all(dir).unwrap();
    }

    #[test]
    fn identificadores_no_validos() {
        assert!(undo("no-es-un-id", false).is_err());
        assert!(undo("otro.log:3", false).is_err());
        assert!(undo("tweaks.log:abc", false).is_err());
    }

    #[test]
    fn codifica_busquedas() {
        assert_eq!(encode_query("El Tiempo"), "El%20Tiempo");
        assert_eq!(encode_query("Películas & TV"), "Pel%C3%ADculas%20%26%20TV");
    }

    /// Lee los logs reales: `cargo test -- --ignored --nocapture`.
    #[test]
    #[ignore]
    fn historial_real() {
        for e in list() {
            println!("{} | {} | {} | ok={} | {:?}", e.time, e.module, e.text, e.ok, e.action);
        }
    }
}
