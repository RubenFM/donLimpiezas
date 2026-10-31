//! Liberar espacio local de OneDrive y Dropbox dejando archivos "solo en la nube".
//!
//! Solo se leen atributos (nunca el contenido, que provocaría descargas) y para
//! liberar se usa `attrib -P +U`, lo mismo que "Liberar espacio" del Explorador.

use crate::log;
use serde::Serialize;
use std::collections::HashSet;
use std::fs;
use std::os::windows::fs::MetadataExt;
use std::os::windows::process::CommandExt;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::Mutex;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use winreg::enums::{HKEY_CURRENT_USER, KEY_READ};
use winreg::RegKey;

const OFFLINE: u32 = 0x1000;
const RECALL_ON_OPEN: u32 = 0x4_0000;
const PINNED: u32 = 0x8_0000;
const UNPINNED: u32 = 0x10_0000;
const RECALL_ON_DATA_ACCESS: u32 = 0x40_0000;
const CREATE_NO_WINDOW: u32 = 0x0800_0000;
const MAX_FILES: usize = 1000;

#[derive(Serialize, Clone)]
pub struct CloudRoot {
    name: String,
    path: String,
}

/// Archivos del último análisis, para validar qué se puede liberar.
#[derive(Default)]
pub struct State {
    last: Mutex<HashSet<String>>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CloudFile {
    path: String,
    root: String,
    size: u64,
    /// Segundos desde 1970.
    modified: Option<u64>,
    pinned: bool,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CloudScan {
    /// Los `MAX_FILES` más grandes que cumplen los filtros.
    files: Vec<CloudFile>,
    total_files: usize,
    matching_bytes: u64,
    /// Todo lo que ocupan en el disco las carpetas analizadas.
    local_bytes: u64,
}

#[derive(Serialize)]
pub struct FreeResult {
    freed: usize,
    bytes: u64,
    failed: Vec<String>,
}

#[derive(PartialEq, Debug)]
enum FileState {
    CloudOnly,
    /// Ya se pidió liberarlo; el proveedor lo hará en segundo plano.
    Unpinned,
    Pinned,
    Local,
}

fn classify(attrs: u32) -> FileState {
    if attrs & (OFFLINE | RECALL_ON_OPEN | RECALL_ON_DATA_ACCESS) != 0 {
        FileState::CloudOnly
    } else if attrs & UNPINNED != 0 {
        FileState::Unpinned
    } else if attrs & PINNED != 0 {
        FileState::Pinned
    } else {
        FileState::Local
    }
}

fn key(path: &Path) -> String {
    path.to_string_lossy().to_lowercase()
}

/// Carpetas de `info.json` de Dropbox: `{"personal": {"path": ...}, "business": {...}}`.
fn parse_dropbox(json: &str) -> Vec<CloudRoot> {
    let Ok(value) = serde_json::from_str::<serde_json::Value>(json) else {
        return Vec::new();
    };
    [("personal", "Dropbox (personal)"), ("business", "Dropbox (empresa)")]
        .into_iter()
        .filter_map(|(account, name)| {
            let path = value.get(account)?.get("path")?.as_str()?;
            Some(CloudRoot { name: name.into(), path: path.into() })
        })
        .collect()
}

fn onedrive_roots() -> Vec<CloudRoot> {
    let Ok(accounts) =
        RegKey::predef(HKEY_CURRENT_USER).open_subkey_with_flags(r"Software\Microsoft\OneDrive\Accounts", KEY_READ)
    else {
        return Vec::new();
    };
    accounts
        .enum_keys()
        .flatten()
        .filter_map(|account| {
            let key = accounts.open_subkey(&account).ok()?;
            let folder: String = key.get_value("UserFolder").ok()?;
            if folder.is_empty() {
                return None;
            }
            let label = if account == "Personal" {
                "personal".to_string()
            } else {
                key.get_value::<String, _>("DisplayName").unwrap_or(account)
            };
            Some(CloudRoot { name: format!("OneDrive ({label})"), path: folder })
        })
        .collect()
}

fn dropbox_roots() -> Vec<CloudRoot> {
    ["LOCALAPPDATA", "APPDATA"]
        .iter()
        .filter_map(|var| std::env::var_os(var))
        .map(|base| PathBuf::from(base).join(r"Dropbox\info.json"))
        .find_map(|file| fs::read_to_string(file).ok())
        .map(|json| parse_dropbox(&json))
        .unwrap_or_default()
}

pub fn roots() -> Vec<CloudRoot> {
    let mut roots = onedrive_roots();
    roots.extend(dropbox_roots());
    roots.retain(|r| Path::new(&r.path).is_dir());
    roots
}

pub fn scan(selected: &[String], min_size: u64, min_age_days: u64, state: &State) -> Result<CloudScan, String> {
    let roots: Vec<CloudRoot> = roots()
        .into_iter()
        .filter(|r| selected.iter().any(|s| s.eq_ignore_ascii_case(&r.path)))
        .collect();
    if roots.is_empty() {
        return Err("Ninguna de las carpetas elegidas es una carpeta sincronizada.".into());
    }
    let cutoff = SystemTime::now() - Duration::from_secs(min_age_days * 86_400);

    let mut files = Vec::new();
    let mut local_bytes = 0;
    for root in &roots {
        let mut stack = vec![PathBuf::from(&root.path)];
        while let Some(dir) = stack.pop() {
            let Ok(entries) = fs::read_dir(&dir) else {
                continue;
            };
            for entry in entries.flatten() {
                let path = entry.path();
                let Ok(meta) = fs::symlink_metadata(&path) else {
                    continue;
                };
                // Las carpetas de OneDrive también son puntos de reanálisis: solo se
                // descartan enlaces y uniones.
                if meta.file_type().is_symlink() {
                    continue;
                }
                if meta.is_dir() {
                    stack.push(path);
                    continue;
                }
                let state = classify(meta.file_attributes());
                if !matches!(state, FileState::Local | FileState::Pinned) {
                    continue;
                }
                local_bytes += meta.len();
                let modified = meta.modified().ok();
                let old_enough = min_age_days == 0 || modified.is_some_and(|m| m <= cutoff);
                if meta.len() >= min_size && old_enough {
                    files.push(CloudFile {
                        path: path.to_string_lossy().into_owned(),
                        root: root.name.clone(),
                        size: meta.len(),
                        modified: modified.and_then(|m| m.duration_since(UNIX_EPOCH).ok()).map(|d| d.as_secs()),
                        pinned: state == FileState::Pinned,
                    });
                }
            }
        }
    }

    files.sort_by_key(|f| std::cmp::Reverse(f.size));
    *state.last.lock().unwrap() = files.iter().map(|f| key(Path::new(&f.path))).collect();
    let total_files = files.len();
    let matching_bytes = files.iter().map(|f| f.size).sum();
    files.truncate(MAX_FILES);
    Ok(CloudScan { files, total_files, matching_bytes, local_bytes })
}

fn validate(paths: &[String], known: &HashSet<String>) -> Result<(), String> {
    match paths.iter().find(|p| !known.contains(&key(Path::new(p)))) {
        Some(p) => Err(format!("«{p}» no está en el último análisis. Vuelve a analizar.")),
        None => Ok(()),
    }
}

pub fn free(paths: &[String], state: &State) -> Result<FreeResult, String> {
    let mut known = state.last.lock().unwrap();
    validate(paths, &known)?;

    let mut result = FreeResult { freed: 0, bytes: 0, failed: Vec::new() };
    for path in paths {
        let size = fs::metadata(path).map(|m| m.len()).unwrap_or(0);
        let _ = Command::new("attrib")
            .args(["-P", "+U", path])
            .creation_flags(CREATE_NO_WINDOW)
            .output();
        // `attrib` no siempre devuelve error al fallar: se comprueba el atributo.
        let freed = fs::symlink_metadata(path)
            .map(|m| matches!(classify(m.file_attributes()), FileState::Unpinned | FileState::CloudOnly))
            .unwrap_or(false);
        if freed {
            result.freed += 1;
            result.bytes += size;
            known.remove(&key(Path::new(path)));
            let _ = log::append("cloud.log", path);
        } else {
            result.failed.push(path.clone());
        }
    }
    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn clasifica_por_atributos() {
        // Archive | ReparsePoint: descargado.
        assert_eq!(classify(0x20 | 0x400), FileState::Local);
        assert_eq!(classify(0x20 | 0x400 | PINNED), FileState::Pinned);
        assert_eq!(classify(0x20 | 0x400 | UNPINNED), FileState::Unpinned);
        assert_eq!(classify(0x20 | 0x400 | UNPINNED | RECALL_ON_DATA_ACCESS), FileState::CloudOnly);
        assert_eq!(classify(OFFLINE), FileState::CloudOnly);
    }

    #[test]
    fn lee_info_json_de_dropbox() {
        let json = r#"{"personal": {"path": "C:\\Users\\ana\\Dropbox", "host": 1},
                      "business": {"path": "C:\\Users\\ana\\Dropbox (Empresa)"}}"#;
        let roots = parse_dropbox(json);
        assert_eq!(roots.len(), 2);
        assert_eq!(roots[0].name, "Dropbox (personal)");
        assert_eq!(roots[0].path, r"C:\Users\ana\Dropbox");
        assert_eq!(roots[1].name, "Dropbox (empresa)");
        assert!(parse_dropbox("no es json").is_empty());
        assert!(parse_dropbox("{}").is_empty());
    }

    #[test]
    fn rechaza_rutas_no_analizadas() {
        let state = State::default();
        state.last.lock().unwrap().insert(key(Path::new(r"C:\Users\ruben\OneDrive\video.mp4")));
        assert!(free(&[r"C:\Windows\explorer.exe".into()], &state).is_err());
        assert!(validate(&[r"c:\users\ruben\onedrive\VIDEO.mp4".into()], &state.last.lock().unwrap()).is_ok());
    }

    /// Solo lee atributos: `cargo test -- --ignored --nocapture`.
    #[test]
    #[ignore]
    fn analiza_onedrive_real() {
        let roots = roots();
        println!("{:?}", roots.iter().map(|r| (&r.name, &r.path)).collect::<Vec<_>>());
        assert!(!roots.is_empty());
        let paths: Vec<String> = roots.iter().map(|r| r.path.clone()).collect();
        let scan = scan(&paths, 10 * 1024 * 1024, 0, &State::default()).unwrap();
        println!(
            "{} MB en local, {} archivos >= 10 MB ({} MB)",
            scan.local_bytes >> 20,
            scan.total_files,
            scan.matching_bytes >> 20
        );
        for f in scan.files.iter().take(5) {
            println!("{} MB {}{}", f.size >> 20, f.path, if f.pinned { " [fijado]" } else { "" });
        }
    }
}
