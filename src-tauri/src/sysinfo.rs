//! Información básica del equipo para Health Check.

use serde::Serialize;
use windows_sys::Win32::Storage::FileSystem::{GetDiskFreeSpaceExW, GetDriveTypeW, GetLogicalDriveStringsW};
use windows_sys::Win32::System::SystemInformation::{GetTickCount64, GlobalMemoryStatusEx, MEMORYSTATUSEX};
use winreg::enums::{HKEY_LOCAL_MACHINE, KEY_READ};
use winreg::RegKey;

const DRIVE_FIXED: u32 = 3;
/// Primera compilación de Windows 11.
const WINDOWS_11_BUILD: u32 = 22000;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Disk {
    letter: String,
    free: u64,
    total: u64,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SystemInfo {
    windows: String,
    uptime_secs: u64,
    memory_total: u64,
    memory_used: u64,
    disks: Vec<Disk>,
}

/// El registro sigue diciendo "Windows 10" en Windows 11.
fn windows_name(product: &str, display_version: Option<&str>, build: Option<u32>) -> String {
    let mut name = product.trim().to_string();
    if build.is_some_and(|b| b >= WINDOWS_11_BUILD) {
        name = name.replace("Windows 10", "Windows 11");
    }
    if let Some(v) = display_version.filter(|v| !v.is_empty()) {
        name = format!("{name} {v}");
    }
    if let Some(b) = build {
        name = format!("{name} (compilación {b})");
    }
    name
}

fn windows() -> String {
    let Ok(key) = RegKey::predef(HKEY_LOCAL_MACHINE)
        .open_subkey_with_flags(r"SOFTWARE\Microsoft\Windows NT\CurrentVersion", KEY_READ)
    else {
        return "Windows".into();
    };
    let text = |name: &str| key.get_value::<String, _>(name).ok();
    windows_name(
        &text("ProductName").unwrap_or_else(|| "Windows".into()),
        text("DisplayVersion").as_deref(),
        text("CurrentBuild").and_then(|b| b.parse().ok()),
    )
}

fn memory() -> (u64, u64) {
    let mut status: MEMORYSTATUSEX = unsafe { std::mem::zeroed() };
    status.dwLength = std::mem::size_of::<MEMORYSTATUSEX>() as u32;
    if unsafe { GlobalMemoryStatusEx(&mut status) } == 0 {
        return (0, 0);
    }
    (status.ullTotalPhys, status.ullTotalPhys - status.ullAvailPhys)
}

/// Espacio libre y total de una unidad (raíz como `C:\`).
fn disk_space(root: &[u16]) -> Option<(u64, u64)> {
    let mut wide: Vec<u16> = root.to_vec();
    wide.push(0);
    let (mut free, mut total) = (0u64, 0u64);
    let ok = unsafe { GetDiskFreeSpaceExW(wide.as_ptr(), &mut free, &mut total, std::ptr::null_mut()) };
    (ok != 0).then_some((free, total))
}

/// Espacio libre del disco del sistema.
pub(crate) fn system_drive_free() -> Option<u64> {
    let drive = std::env::var("SystemDrive").unwrap_or_else(|_| "C:".into());
    let root: Vec<u16> = format!("{drive}\\").encode_utf16().collect();
    disk_space(&root).map(|(free, _)| free)
}

fn fixed_disks() -> Vec<Disk> {
    let mut buf = [0u16; 512];
    let len = unsafe { GetLogicalDriveStringsW(buf.len() as u32, buf.as_mut_ptr()) } as usize;
    // "C:\\\0D:\\\0\0": raíces separadas por nulos.
    buf[..len.min(buf.len())]
        .split(|&c| c == 0)
        .filter(|root| !root.is_empty())
        .filter_map(|root| {
            let mut wide: Vec<u16> = root.to_vec();
            wide.push(0);
            if unsafe { GetDriveTypeW(wide.as_ptr()) } != DRIVE_FIXED {
                return None;
            }
            let (free, total) = disk_space(root)?;
            Some(Disk { letter: String::from_utf16_lossy(root).trim_end_matches('\\').to_string(), free, total })
        })
        .collect()
}

pub fn info() -> SystemInfo {
    let (memory_total, memory_used) = memory();
    SystemInfo {
        windows: windows(),
        uptime_secs: unsafe { GetTickCount64() } / 1000,
        memory_total,
        memory_used,
        disks: fixed_disks(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn nombre_de_windows() {
        assert_eq!(
            windows_name("Windows 10 Pro", Some("24H2"), Some(26100)),
            "Windows 11 Pro 24H2 (compilación 26100)"
        );
        assert_eq!(
            windows_name("Windows 10 Home", Some("22H2"), Some(19045)),
            "Windows 10 Home 22H2 (compilación 19045)"
        );
        assert_eq!(windows_name("Windows 10 Pro", None, None), "Windows 10 Pro");
    }

    /// Datos reales: `cargo test -- --ignored --nocapture`.
    #[test]
    #[ignore]
    fn datos_reales() {
        let info = info();
        println!(
            "{} | encendido {} h | RAM {}/{} GB",
            info.windows,
            info.uptime_secs / 3600,
            info.memory_used >> 30,
            info.memory_total >> 30
        );
        for d in &info.disks {
            println!("{} {} GB libres de {} GB", d.letter, d.free >> 30, d.total >> 30);
        }
        assert!(info.memory_total > 0 && !info.disks.is_empty());
    }
}
