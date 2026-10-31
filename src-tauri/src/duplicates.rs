//! Búsqueda de archivos duplicados por contenido y envío a la papelera.

use crate::{log, ps};
use serde::Serialize;
use std::collections::{HashMap, HashSet};
use std::ffi::OsStr;
use std::fs::{self, File};
use std::io::{self, Read};
use std::os::windows::ffi::OsStrExt;
use std::os::windows::fs::MetadataExt;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Mutex;
use std::time::{Duration, Instant, UNIX_EPOCH};
use windows_sys::Win32::UI::Shell::{
    SHFileOperationW, FOF_ALLOWUNDO, FOF_NOCONFIRMATION, FOF_NOERRORUI, FOF_SILENT, FOF_WANTNUKEWARNING,
    FO_DELETE, SHFILEOPSTRUCTW,
};

/// Archivos que solo están en la nube: leerlos provocaría su descarga.
const CLOUD_ONLY: u32 = 0x1000 /* OFFLINE */ | 0x4_0000 /* RECALL_ON_OPEN */ | 0x40_0000 /* RECALL_ON_DATA_ACCESS */;
const PARTIAL_BYTES: usize = 64 * 1024;
const MAX_GROUPS: usize = 300;
const CANCELLED: &str = "Análisis cancelado.";

/// Estado compartido entre comandos: cancelación y grupos del último análisis.
#[derive(Default)]
pub struct State {
    cancel: AtomicBool,
    last: Mutex<Vec<Vec<PathBuf>>>,
}

impl State {
    pub fn cancel(&self) {
        self.cancel.store(true, Ordering::Relaxed);
    }
}

#[derive(Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Progress {
    /// `scanning`, `comparing` o `verifying`.
    phase: &'static str,
    found: u64,
    processed: u64,
    total: u64,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DuplicateFile {
    path: String,
    /// Segundos desde 1970.
    modified: Option<u64>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Group {
    size: u64,
    files: Vec<DuplicateFile>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ScanResult {
    /// Los `MAX_GROUPS` grupos con más espacio recuperable.
    groups: Vec<Group>,
    total_groups: usize,
    /// Espacio recuperable conservando una copia de cada grupo.
    wasted: u64,
}

#[derive(Serialize)]
pub struct TrashResult {
    trashed: usize,
}

/// Envía progreso como mucho cada 100 ms (y siempre la primera vez).
struct Reporter<F: FnMut(Progress)> {
    send: F,
    last: Option<Instant>,
    progress: Progress,
}

impl<F: FnMut(Progress)> Reporter<F> {
    fn tick(&mut self) {
        if self.last.is_none_or(|t| t.elapsed() >= Duration::from_millis(100)) {
            self.last = Some(Instant::now());
            (self.send)(self.progress.clone());
        }
    }

    fn phase(&mut self, phase: &'static str, total: u64) {
        self.progress.phase = phase;
        self.progress.processed = 0;
        self.progress.total = total;
        self.last = None;
        self.tick();
    }
}

fn check(cancel: &AtomicBool) -> Result<(), String> {
    if cancel.load(Ordering::Relaxed) {
        Err(CANCELLED.into())
    } else {
        Ok(())
    }
}

fn key(path: &Path) -> String {
    path.to_string_lossy().to_lowercase()
}

/// Quita carpetas repetidas o contenidas en otra de la lista.
fn dedupe_roots(folders: &[String]) -> Vec<PathBuf> {
    let mut sorted: Vec<&String> = folders.iter().collect();
    sorted.sort_by_key(|f| f.len());
    let mut roots: Vec<PathBuf> = Vec::new();
    for folder in sorted {
        let path = PathBuf::from(folder.trim_end_matches('\\'));
        let k = key(&path);
        let inside = roots.iter().any(|r| {
            let rk = key(r);
            k == rk || k.starts_with(&format!("{rk}\\"))
        });
        if !inside {
            roots.push(path);
        }
    }
    roots
}

fn collect(
    roots: &[PathBuf],
    min_size: u64,
    cancel: &AtomicBool,
    reporter: &mut Reporter<impl FnMut(Progress)>,
) -> Result<HashMap<u64, Vec<PathBuf>>, String> {
    let mut by_size: HashMap<u64, Vec<PathBuf>> = HashMap::new();
    let mut stack: Vec<PathBuf> = roots.to_vec();
    while let Some(dir) = stack.pop() {
        check(cancel)?;
        let Ok(entries) = fs::read_dir(&dir) else {
            continue;
        };
        for entry in entries.flatten() {
            let path = entry.path();
            let Ok(meta) = fs::symlink_metadata(&path) else {
                continue;
            };
            // Enlaces y uniones nunca se siguen. Los archivos de OneDrive también son
            // puntos de reanálisis, así que no se descartan solo por serlo.
            if meta.file_type().is_symlink() {
                continue;
            }
            if meta.is_dir() {
                stack.push(path);
            } else if meta.file_attributes() & CLOUD_ONLY == 0 && meta.len() >= min_size.max(1) {
                by_size.entry(meta.len()).or_default().push(path);
                reporter.progress.found += 1;
                reporter.tick();
            }
        }
    }
    by_size.retain(|_, files| files.len() > 1);
    Ok(by_size)
}

/// Hash BLAKE3 de los primeros `limit` bytes (o de todo el archivo).
fn hash_file(path: &Path, limit: Option<usize>, cancel: &AtomicBool) -> io::Result<[u8; 32]> {
    let mut file = File::open(path)?;
    let mut hasher = blake3::Hasher::new();
    let mut buf = vec![0u8; 256 * 1024];
    let mut remaining = limit.unwrap_or(usize::MAX);
    while remaining > 0 {
        if cancel.load(Ordering::Relaxed) {
            return Err(io::Error::new(io::ErrorKind::Interrupted, CANCELLED));
        }
        let want = remaining.min(buf.len());
        let n = file.read(&mut buf[..want])?;
        if n == 0 {
            break;
        }
        hasher.update(&buf[..n]);
        remaining -= n;
    }
    Ok(*hasher.finalize().as_bytes())
}

/// Subdivide cada grupo por hash; descarta los que se quedan con un solo archivo.
fn split_by_hash(
    groups: Vec<(u64, Vec<PathBuf>)>,
    limit: Option<usize>,
    cancel: &AtomicBool,
    reporter: &mut Reporter<impl FnMut(Progress)>,
) -> Result<Vec<(u64, Vec<PathBuf>)>, String> {
    let mut out = Vec::new();
    for (size, files) in groups {
        let mut by_hash: HashMap<[u8; 32], Vec<PathBuf>> = HashMap::new();
        for path in files {
            match hash_file(&path, limit, cancel) {
                Ok(hash) => by_hash.entry(hash).or_default().push(path),
                Err(e) if e.kind() == io::ErrorKind::Interrupted => return Err(CANCELLED.into()),
                // Archivos bloqueados o sin permiso: se ignoran.
                Err(_) => {}
            }
            reporter.progress.processed += 1;
            reporter.tick();
        }
        out.extend(by_hash.into_values().filter(|f| f.len() > 1).map(|f| (size, f)));
    }
    Ok(out)
}

fn modified_secs(path: &Path) -> Option<u64> {
    let modified = fs::metadata(path).ok()?.modified().ok()?;
    Some(modified.duration_since(UNIX_EPOCH).ok()?.as_secs())
}

pub fn find(
    folders: &[String],
    min_size: u64,
    state: &State,
    send: impl FnMut(Progress),
) -> Result<ScanResult, String> {
    let cancel = &state.cancel;
    cancel.store(false, Ordering::Relaxed);
    let mut reporter = Reporter {
        send,
        last: None,
        progress: Progress { phase: "scanning", found: 0, processed: 0, total: 0 },
    };
    reporter.tick();

    let by_size = collect(&dedupe_roots(folders), min_size, cancel, &mut reporter)?;
    let candidates: Vec<(u64, Vec<PathBuf>)> = by_size.into_iter().collect();

    // Primeros 64 KB; si el archivo no es mayor, ese hash ya es el del contenido completo.
    reporter.phase("comparing", candidates.iter().map(|(_, f)| f.len() as u64).sum());
    let partial = split_by_hash(candidates, Some(PARTIAL_BYTES), cancel, &mut reporter)?;
    let (small, large): (Vec<_>, Vec<_>) = partial.into_iter().partition(|(size, _)| *size <= PARTIAL_BYTES as u64);

    reporter.phase("verifying", large.iter().map(|(_, f)| f.len() as u64).sum());
    let mut groups = split_by_hash(large, None, cancel, &mut reporter)?;
    groups.extend(small);

    let wasted_of = |(size, files): &(u64, Vec<PathBuf>)| size * (files.len() as u64 - 1);
    groups.sort_by_key(|g| std::cmp::Reverse(wasted_of(g)));
    let wasted = groups.iter().map(wasted_of).sum();
    let total_groups = groups.len();

    let result = ScanResult {
        groups: groups
            .iter()
            .take(MAX_GROUPS)
            .map(|(size, files)| Group {
                size: *size,
                files: files
                    .iter()
                    .map(|p| DuplicateFile { path: p.to_string_lossy().into_owned(), modified: modified_secs(p) })
                    .collect(),
            })
            .collect(),
        total_groups,
        wasted,
    };
    *state.last.lock().unwrap() = groups.into_iter().map(|(_, files)| files).collect();
    Ok(result)
}

/// Comprueba que todas las rutas son del último análisis y que ningún grupo
/// se queda sin copias.
fn validate(groups: &[Vec<PathBuf>], paths: &[String]) -> Result<(), String> {
    let selected: HashSet<String> = paths.iter().map(|p| key(Path::new(p))).collect();
    let known: HashSet<String> = groups.iter().flatten().map(|p| key(p)).collect();
    if let Some(unknown) = paths.iter().find(|p| !known.contains(&key(Path::new(p)))) {
        return Err(format!("«{unknown}» no está en el último análisis. Vuelve a analizar."));
    }
    if groups.iter().any(|g| g.iter().all(|p| selected.contains(&key(p)))) {
        return Err("Debes conservar al menos una copia de cada grupo.".into());
    }
    Ok(())
}

fn wide_list(paths: &[&String]) -> Vec<u16> {
    // Lista de rutas separadas por nulos y terminada en doble nulo.
    let mut out: Vec<u16> = paths
        .iter()
        .flat_map(|p| OsStr::new(p.as_str()).encode_wide().chain(Some(0)))
        .collect();
    out.push(0);
    out
}

pub fn trash(paths: &[String], state: &State) -> Result<TrashResult, String> {
    let mut groups = state.last.lock().unwrap();
    validate(&groups, paths)?;
    let existing: Vec<&String> = paths.iter().filter(|p| Path::new(p).is_file()).collect();
    if existing.is_empty() {
        return Ok(TrashResult { trashed: 0 });
    }

    let from = wide_list(&existing);
    let mut op = SHFILEOPSTRUCTW {
        hwnd: std::ptr::null_mut(),
        wFunc: FO_DELETE,
        pFrom: from.as_ptr(),
        pTo: std::ptr::null(),
        // Avisa si algún archivo no cabe en la papelera y se borraría para siempre.
        fFlags: (FOF_ALLOWUNDO | FOF_NOCONFIRMATION | FOF_SILENT | FOF_NOERRORUI | FOF_WANTNUKEWARNING) as u16,
        fAnyOperationsAborted: 0,
        hNameMappings: std::ptr::null_mut(),
        lpszProgressTitle: std::ptr::null(),
    };
    let code = unsafe { SHFileOperationW(&mut op) };
    if code != 0 {
        return Err(format!("No se pudieron enviar a la papelera (código {code})."));
    }
    if op.fAnyOperationsAborted != 0 {
        return Err("Se canceló el envío a la papelera.".into());
    }

    let trashed: HashSet<String> = existing.iter().filter(|p| !Path::new(p).exists()).map(|p| key(Path::new(p))).collect();
    for p in existing.iter().filter(|p| trashed.contains(&key(Path::new(p)))) {
        let _ = log::append("duplicates.log", p);
    }
    for group in groups.iter_mut() {
        group.retain(|p| !trashed.contains(&key(p)));
    }
    groups.retain(|g| g.len() > 1);
    Ok(TrashResult { trashed: trashed.len() })
}

/// Documentos, Descargas, Imágenes, Vídeos y Música en su ubicación real.
pub fn default_folders() -> Result<Vec<String>, String> {
    let json = ps::run(
        "$paths = @(
            [Environment]::GetFolderPath('MyDocuments'),
            (New-Object -ComObject Shell.Application).NameSpace('shell:Downloads').Self.Path,
            [Environment]::GetFolderPath('MyPictures'),
            [Environment]::GetFolderPath('MyVideos'),
            [Environment]::GetFolderPath('MyMusic')
        )
        ConvertTo-Json -Compress -InputObject @($paths | Where-Object { $_ -and (Test-Path -LiteralPath $_) })",
    )?;
    serde_json::from_str(&json).map_err(|e| format!("Respuesta inesperada de PowerShell: {e}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    struct TempDir(PathBuf);

    impl TempDir {
        fn new(name: &str) -> Self {
            let dir = std::env::temp_dir().join(format!("donlimpiezas-{name}-{}", std::process::id()));
            let _ = fs::remove_dir_all(&dir);
            fs::create_dir_all(dir.join("sub")).unwrap();
            TempDir(dir)
        }

        fn file(&self, name: &str, content: &[u8]) -> PathBuf {
            let path = self.0.join(name);
            fs::write(&path, content).unwrap();
            path
        }
    }

    impl Drop for TempDir {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.0);
        }
    }

    fn sorted(mut paths: Vec<String>) -> Vec<String> {
        paths.sort();
        paths
    }

    #[test]
    fn encuentra_duplicados_por_contenido() {
        let dir = TempDir::new("dups");
        let big = vec![7u8; 100 * 1024];
        let mut big_other = big.clone();
        *big_other.last_mut().unwrap() = 8; // Mismos primeros 64 KB, distinto final.

        let a = dir.file("a.bin", &big);
        let copy = dir.file(r"sub\copia con otro nombre.bin", &big);
        dir.file("distinto.bin", &big_other);
        let t1 = dir.file("t1.txt", b"veinte bytes iguales");
        let t2 = dir.file(r"sub\t2.txt", b"veinte bytes iguales");
        dir.file("p1.txt", b"corto");
        dir.file("p2.txt", b"corto");

        let state = State::default();
        let folder = dir.0.to_string_lossy().into_owned();
        // La carpeta anidada repetida no debe duplicar resultados.
        let folders = [folder.clone(), format!(r"{folder}\sub")];
        let result = find(&folders, 10, &state, |_| {}).unwrap();

        let groups: Vec<Vec<String>> =
            result.groups.iter().map(|g| sorted(g.files.iter().map(|f| f.path.clone()).collect())).collect();
        let lossy = |p: &PathBuf| p.to_string_lossy().into_owned();
        assert_eq!(groups.len(), 2, "{groups:?}");
        assert_eq!(groups[0], sorted(vec![lossy(&a), lossy(&copy)]));
        assert_eq!(groups[1], sorted(vec![lossy(&t1), lossy(&t2)]));
        assert_eq!(result.wasted, 100 * 1024 + 20);
        assert_eq!(state.last.lock().unwrap().len(), 2);
    }

    #[test]
    fn cancelar_devuelve_error() {
        let dir = TempDir::new("cancel");
        dir.file("a.txt", b"contenido repetido");
        dir.file("b.txt", b"contenido repetido");
        let state = State::default();
        let folder = dir.0.to_string_lossy().into_owned();
        let result = find(&[folder], 1, &state, |_| state.cancel());
        assert_eq!(result.err().as_deref(), Some(CANCELLED));
    }

    #[test]
    fn validacion_antes_de_borrar() {
        let groups = vec![vec![PathBuf::from(r"C:\a\x.jpg"), PathBuf::from(r"C:\b\x.jpg")]];
        assert!(validate(&groups, &[r"C:\a\x.jpg".into()]).is_ok());
        // Mayúsculas distintas: misma ruta en Windows.
        assert!(validate(&groups, &[r"c:\A\X.JPG".into()]).is_ok());
        assert!(validate(&groups, &[r"C:\a\x.jpg".into(), r"C:\b\x.jpg".into()]).is_err());
        assert!(validate(&groups, &[r"C:\Windows\explorer.exe".into()]).is_err());
    }

    #[test]
    fn trash_rechaza_sin_borrar() {
        let dir = TempDir::new("trash");
        let a = dir.file("a.txt", b"contenido repetido");
        let b = dir.file("b.txt", b"contenido repetido");
        let state = State::default();
        *state.last.lock().unwrap() = vec![vec![a.clone(), b.clone()]];
        let all = [a.to_string_lossy().into_owned(), b.to_string_lossy().into_owned()];
        assert!(trash(&all, &state).is_err());
        assert!(a.exists() && b.exists());
    }

    /// Analiza las carpetas reales sin borrar nada: `cargo test -- --ignored --nocapture`.
    #[test]
    #[ignore]
    fn analiza_carpetas_reales() {
        let started = Instant::now();
        let state = State::default();
        let mut last = None;
        let result = find(&default_folders().unwrap(), 1024 * 1024, &state, |p| last = Some(p)).unwrap();
        let last = last.unwrap();
        println!(
            "{} archivos, {} grupos, {} MB recuperables, {:?}",
            last.found,
            result.total_groups,
            result.wasted / 1_048_576,
            started.elapsed()
        );
        for g in result.groups.iter().take(5) {
            println!("{} KB x{}: {}", g.size / 1024, g.files.len(), g.files[0].path);
        }
    }

    #[test]
    #[ignore]
    fn carpetas_por_defecto_existen() {
        let folders = default_folders().unwrap();
        println!("{folders:?}");
        assert!(!folders.is_empty());
        assert!(folders.iter().all(|f| Path::new(f).is_dir()));
    }
}
