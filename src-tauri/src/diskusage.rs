//! Análisis del espacio ocupado por carpetas. Solo lee; nunca borra nada.

use crate::system;
use serde::Serialize;
use std::collections::HashMap;
use std::fs;
use std::os::windows::fs::MetadataExt;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Mutex;
use std::time::{Duration, Instant};

/// Archivos que solo están en la nube: no ocupan espacio en el disco.
const CLOUD_ONLY: u32 = 0x1000 /* OFFLINE */ | 0x4_0000 /* RECALL_ON_OPEN */ | 0x40_0000 /* RECALL_ON_DATA_ACCESS */;
const TOP_FILES: usize = 20;
const CANCELLED: &str = "Análisis cancelado.";

struct Node {
    path: String,
    size: u64,
    /// Subcarpetas con su tamaño total.
    dirs: Vec<(String, u64)>,
    /// Los archivos más grandes de esta carpeta (no de las subcarpetas).
    top_files: Vec<(String, u64)>,
    other_files: u64,
    other_bytes: u64,
}

/// Resultado del último análisis, para navegar sin volver a leer el disco.
#[derive(Default)]
pub struct State {
    cancel: AtomicBool,
    nodes: Mutex<HashMap<String, Node>>,
}

impl State {
    pub fn cancel(&self) {
        self.cancel.store(true, Ordering::Relaxed);
    }
}

#[derive(Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Progress {
    files: u64,
    dirs: u64,
    current: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Summary {
    root: String,
    size: u64,
    files: u64,
    dirs: u64,
    /// Carpetas que no se pudieron leer (normalmente por permisos).
    unreadable: u64,
    secs: f64,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Item {
    name: String,
    /// `dir`, `file` u `other` (resto de archivos agrupados).
    kind: &'static str,
    /// Solo en carpetas: para entrar en ellas.
    path: Option<String>,
    size: u64,
    /// En `other`: cuántos archivos agrupa.
    count: Option<u64>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Folder {
    path: String,
    size: u64,
    items: Vec<Item>,
}

fn key(path: &str) -> String {
    path.trim_end_matches('\\').to_lowercase()
}

struct Walker<'a, F: FnMut(Progress)> {
    cancel: &'a AtomicBool,
    nodes: HashMap<String, Node>,
    files: u64,
    dirs: u64,
    unreadable: u64,
    send: F,
    last: Option<Instant>,
}

impl<F: FnMut(Progress)> Walker<'_, F> {
    fn tick(&mut self, current: &Path) {
        if self.last.is_none_or(|t| t.elapsed() >= Duration::from_millis(150)) {
            self.last = Some(Instant::now());
            (self.send)(Progress { files: self.files, dirs: self.dirs, current: current.to_string_lossy().into_owned() });
        }
    }

    /// Recorre `path` y devuelve su tamaño total; guarda un nodo por carpeta.
    fn walk(&mut self, path: &Path) -> Result<u64, String> {
        if self.cancel.load(Ordering::Relaxed) {
            return Err(CANCELLED.into());
        }
        let display = path.to_string_lossy().into_owned();
        let mut node = Node { path: display, size: 0, dirs: Vec::new(), top_files: Vec::new(), other_files: 0, other_bytes: 0 };
        let Ok(entries) = fs::read_dir(path) else {
            self.unreadable += 1;
            self.nodes.insert(key(&node.path), node);
            return Ok(0);
        };
        self.tick(path);
        let mut files = Vec::new();
        for entry in entries.flatten() {
            // En Windows, tipo y metadatos vienen del propio listado: no se abre cada archivo.
            let Ok(file_type) = entry.file_type() else {
                continue;
            };
            // Enlaces y uniones nunca se siguen (OneDrive usa otros puntos de reanálisis).
            if file_type.is_symlink() {
                continue;
            }
            let name = entry.file_name().to_string_lossy().into_owned();
            if file_type.is_dir() {
                self.dirs += 1;
                let size = self.walk(&entry.path())?;
                node.size += size;
                node.dirs.push((name, size));
            } else if let Ok(meta) = entry.metadata() {
                let size = if meta.file_attributes() & CLOUD_ONLY != 0 { 0 } else { meta.len() };
                self.files += 1;
                node.size += size;
                files.push((name, size));
            }
        }
        node.dirs.sort_by_key(|(_, s)| std::cmp::Reverse(*s));
        files.sort_by_key(|(_, s)| std::cmp::Reverse(*s));
        for (name, size) in files {
            if node.top_files.len() < TOP_FILES {
                node.top_files.push((name, size));
            } else {
                node.other_files += 1;
                node.other_bytes += size;
            }
        }
        let size = node.size;
        self.nodes.insert(key(&node.path), node);
        Ok(size)
    }
}

pub fn analyze(root: &str, state: &State, send: impl FnMut(Progress)) -> Result<Summary, String> {
    let root_path = PathBuf::from(root);
    if !root_path.is_dir() {
        return Err(format!("«{root}» no es una carpeta."));
    }
    state.cancel.store(false, Ordering::Relaxed);
    let started = Instant::now();
    let mut walker = Walker { cancel: &state.cancel, nodes: HashMap::new(), files: 0, dirs: 0, unreadable: 0, send, last: None };
    let size = walker.walk(&root_path)?;
    let summary = Summary {
        root: root_path.to_string_lossy().into_owned(),
        size,
        files: walker.files,
        dirs: walker.dirs,
        unreadable: walker.unreadable,
        secs: started.elapsed().as_secs_f64(),
    };
    *state.nodes.lock().unwrap() = walker.nodes;
    Ok(summary)
}

pub fn children(path: &str, state: &State) -> Result<Folder, String> {
    let nodes = state.nodes.lock().unwrap();
    let node = nodes.get(&key(path)).ok_or("Esa carpeta no está en el último análisis.")?;
    let base = Path::new(&node.path);
    let mut items: Vec<Item> = node
        .dirs
        .iter()
        .map(|(name, size)| Item {
            name: name.clone(),
            kind: "dir",
            path: Some(base.join(name).to_string_lossy().into_owned()),
            size: *size,
            count: None,
        })
        .chain(node.top_files.iter().map(|(name, size)| Item {
            name: name.clone(),
            kind: "file",
            path: None,
            size: *size,
            count: None,
        }))
        .collect();
    if node.other_files > 0 {
        items.push(Item {
            name: "Otros archivos".into(),
            kind: "other",
            path: None,
            size: node.other_bytes,
            count: Some(node.other_files),
        });
    }
    items.sort_by_key(|i| std::cmp::Reverse(i.size));
    Ok(Folder { path: node.path.clone(), size: node.size, items })
}

pub fn open_in_explorer(path: &str, state: &State) -> Result<(), String> {
    let folder = state
        .nodes
        .lock()
        .unwrap()
        .get(&key(path))
        .map(|n| n.path.clone())
        .ok_or("Esa carpeta no está en el último análisis.")?;
    system::shell_open(&folder, "explore")
}

#[cfg(test)]
mod tests {
    use super::*;

    struct TempDir(PathBuf);

    impl Drop for TempDir {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.0);
        }
    }

    fn temp(name: &str) -> TempDir {
        let dir = std::env::temp_dir().join(format!("donlimpiezas-disk-{name}-{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        TempDir(dir)
    }

    fn write(dir: &Path, name: &str, bytes: usize) {
        let path = dir.join(name);
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(path, vec![0u8; bytes]).unwrap();
    }

    #[test]
    fn suma_tamanos_por_carpeta() {
        let dir = temp("sizes");
        write(&dir.0, r"a\grande.bin", 100);
        write(&dir.0, r"a\b\medio.bin", 50);
        write(&dir.0, r"c\pequeno.bin", 10);
        write(&dir.0, "suelto.txt", 5);
        for i in 0..25 {
            write(&dir.0, &format!(r"muchos\f{i:02}.txt"), i + 1);
        }

        let state = State::default();
        let root = dir.0.to_string_lossy().into_owned();
        let summary = analyze(&root, &state, |_| {}).unwrap();
        let many: u64 = (1..=25).sum();
        assert_eq!(summary.size, 165 + many);
        assert_eq!(summary.files, 29);
        assert_eq!(summary.dirs, 4);
        assert_eq!(summary.unreadable, 0);

        let top = children(&root, &state).unwrap();
        let names: Vec<(&str, u64)> = top.items.iter().map(|i| (i.name.as_str(), i.size)).collect();
        assert_eq!(names, vec![("muchos", many), ("a", 150), ("c", 10), ("suelto.txt", 5)]);

        // 20 archivos más grandes y el resto agrupado.
        let folder = children(&format!(r"{root}\MUCHOS"), &state).unwrap();
        assert_eq!(folder.items.len(), 21);
        let other = folder.items.iter().find(|i| i.kind == "other").unwrap();
        assert_eq!((other.count, other.size), (Some(5), 1 + 2 + 3 + 4 + 5));
    }

    #[test]
    fn rutas_fuera_del_analisis() {
        let state = State::default();
        assert!(children(r"C:\Windows", &state).is_err());
        assert!(open_in_explorer(r"C:\Windows", &state).is_err());
    }

    #[test]
    fn cancelar() {
        let dir = temp("cancel");
        write(&dir.0, r"a\x.bin", 1);
        let state = State::default();
        let result = analyze(&dir.0.to_string_lossy(), &state, |_| state.cancel());
        assert_eq!(result.err().as_deref(), Some(CANCELLED));
    }

    /// Analiza la carpeta del usuario: `cargo test -- --ignored --nocapture`.
    #[test]
    #[ignore]
    fn analiza_carpeta_de_usuario() {
        let home = std::env::var("USERPROFILE").unwrap();
        let state = State::default();
        let s = analyze(&home, &state, |_| {}).unwrap();
        println!(
            "{} GB, {} archivos, {} carpetas, {} ilegibles, {:.1} s",
            s.size >> 30,
            s.files,
            s.dirs,
            s.unreadable,
            s.secs
        );
        for item in children(&home, &state).unwrap().items.iter().take(8) {
            println!("{:>8} MB  {} ({})", item.size >> 20, item.name, item.kind);
        }
    }
}
