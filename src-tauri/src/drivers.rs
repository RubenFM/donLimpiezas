//! Inventario de controladores y actualización exclusivamente con Windows Update.

use crate::{log, ps, system};
use serde::{Deserialize, Serialize};
use std::sync::Mutex;

const SEARCH: &str = "IsInstalled=0 and Type='Driver' and IsHidden=0";

/// Resultado de la última búsqueda, para validar qué se puede instalar.
#[derive(Default)]
pub struct State {
    last: Mutex<Vec<DriverUpdate>>,
}

#[derive(Serialize, Deserialize)]
pub struct Driver {
    device: String,
    class: Option<String>,
    provider: Option<String>,
    version: Option<String>,
    /// `yyyy-mm-dd`.
    date: Option<String>,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct DriverUpdate {
    id: String,
    title: String,
    class: Option<String>,
    manufacturer: Option<String>,
    date: Option<String>,
    size: Option<u64>,
}

#[derive(Serialize, Deserialize)]
pub struct InstallItem {
    id: String,
    title: String,
    ok: bool,
    /// `ResultCode` de Windows Update (2 correcto, 3 correcto con errores, 4 fallo, 5 cancelado).
    code: i32,
}

#[derive(Serialize, Deserialize)]
pub struct InstallResult {
    results: Vec<InstallItem>,
    reboot: bool,
}

fn from_json<T: for<'de> Deserialize<'de>>(json: &str) -> Result<T, String> {
    serde_json::from_str(json).map_err(|e| format!("Respuesta inesperada de PowerShell: {e}"))
}

pub fn list() -> Result<Vec<Driver>, String> {
    let json = ps::run(
        "$d = Get-CimInstance Win32_PnPSignedDriver |
            Where-Object { $_.DeviceName -and $_.DriverVersion } |
            Sort-Object DeviceClass, DeviceName |
            ForEach-Object { [pscustomobject]@{
                device = $_.DeviceName
                class = $_.DeviceClass
                provider = $_.DriverProviderName
                version = $_.DriverVersion
                date = $(if ($_.DriverDate) { $_.DriverDate.ToString('yyyy-MM-dd') } else { $null })
            } }
        ConvertTo-Json -Compress -InputObject @($d)",
    )?;
    from_json(&json)
}

pub fn search(state: &State) -> Result<Vec<DriverUpdate>, String> {
    let json = ps::run(&format!(
        "$s = New-Object -ComObject Microsoft.Update.Session
        $r = $s.CreateUpdateSearcher().Search(\"{SEARCH}\")
        $u = foreach ($x in $r.Updates) {{ [pscustomobject]@{{
            id = $x.Identity.UpdateID
            title = $x.Title
            class = $x.DriverClass
            manufacturer = $x.DriverManufacturer
            date = $(if ($x.DriverVerDate) {{ $x.DriverVerDate.ToString('yyyy-MM-dd') }} else {{ $null }})
            size = [int64]$x.MaxDownloadSize
        }} }}
        ConvertTo-Json -Compress -InputObject @($u)"
    ))
    .map_err(|e| format!("Windows Update no respondió: {e}"))?;
    let updates: Vec<DriverUpdate> = from_json(&json)?;
    *state.last.lock().unwrap() = updates.clone();
    Ok(updates)
}

/// `xxxxxxxx-xxxx-xxxx-xxxx-xxxxxxxxxxxx` en hexadecimal.
fn is_guid(id: &str) -> bool {
    id.len() == 36
        && id.char_indices().all(|(i, c)| {
            if matches!(i, 8 | 13 | 18 | 23) {
                c == '-'
            } else {
                c.is_ascii_hexdigit()
            }
        })
}

/// Solo GUIDs de la última búsqueda: así también es seguro incrustarlos en el script.
fn validate(ids: &[String], last: &[DriverUpdate]) -> Result<(), String> {
    if ids.is_empty() {
        return Err("No hay controladores seleccionados.".into());
    }
    match ids.iter().find(|id| !is_guid(id) || !last.iter().any(|u| &u.id == *id)) {
        Some(id) => Err(format!("«{id}» no está en la última búsqueda. Vuelve a buscar.")),
        None => Ok(()),
    }
}

pub fn install(ids: &[String], is_admin: bool, state: &State) -> Result<InstallResult, String> {
    if !is_admin {
        return Err("Instalar controladores requiere administrador.".into());
    }
    validate(ids, &state.last.lock().unwrap())?;
    let list = ids.iter().map(|id| ps::quote(id)).collect::<Vec<_>>().join(",");
    let json = ps::run(&format!(
        "$ids = @({list})
        $s = New-Object -ComObject Microsoft.Update.Session
        $r = $s.CreateUpdateSearcher().Search(\"{SEARCH}\")
        $coll = New-Object -ComObject Microsoft.Update.UpdateColl
        foreach ($x in $r.Updates) {{
            if ($ids -contains $x.Identity.UpdateID) {{
                if (-not $x.EulaAccepted) {{ $x.AcceptEula() }}
                [void]$coll.Add($x)
            }}
        }}
        if ($coll.Count -eq 0) {{ throw 'Los controladores seleccionados ya no están disponibles.' }}
        $d = $s.CreateUpdateDownloader(); $d.Updates = $coll; [void]$d.Download()
        $i = $s.CreateUpdateInstaller(); $i.Updates = $coll; $res = $i.Install()
        $out = for ($k = 0; $k -lt $coll.Count; $k++) {{
            $code = $res.GetUpdateResult($k).ResultCode
            [pscustomobject]@{{
                id = $coll.Item($k).Identity.UpdateID
                title = $coll.Item($k).Title
                ok = ($code -eq 2 -or $code -eq 3)
                code = [int]$code
            }}
        }}
        ConvertTo-Json -Compress -Depth 3 -InputObject @{{ results = @($out); reboot = [bool]$res.RebootRequired }}"
    ))?;
    let result: InstallResult = from_json(&json)?;
    for item in &result.results {
        let state = if item.ok { "instalado".to_string() } else { format!("ERROR código {}", item.code) };
        let _ = log::append("drivers.log", &format!("{}\t{state}", item.title));
    }
    Ok(result)
}

fn vendor_url(vendor: &str) -> Option<&'static str> {
    match vendor {
        "nvidia" => Some("https://www.nvidia.com/es-es/software/nvidia-app/"),
        "amd" => Some("https://www.amd.com/es/support/download/drivers.html"),
        "intel" => Some("https://www.intel.es/content/www/es/es/support/detect.html"),
        _ => None,
    }
}

/// Abre en el navegador la página oficial de controladores de un fabricante de la lista.
pub fn open_vendor_page(vendor: &str) -> Result<(), String> {
    let url = vendor_url(vendor).ok_or_else(|| format!("Fabricante desconocido: {vendor}"))?;
    system::shell_open(url, "open").map_err(|_| "No se pudo abrir el navegador.".into())
}

#[cfg(test)]
mod tests {
    use super::*;

    const GUID: &str = "8f0c3a1e-5b2d-4e6f-9a7b-1c2d3e4f5a6b";

    fn update(id: &str) -> DriverUpdate {
        DriverUpdate { id: id.into(), title: "Realtek Audio".into(), class: None, manufacturer: None, date: None, size: None }
    }

    #[test]
    fn guids() {
        assert!(is_guid(GUID));
        assert!(!is_guid("8f0c3a1e-5b2d-4e6f-9a7b-1c2d3e4f5a6"));
        assert!(!is_guid("8f0c3a1e-5b2d-4e6f-9a7b-1c2d3e4f5a6'"));
        assert!(!is_guid("'; Remove-Item C:\\ -Recurse; '"));
    }

    #[test]
    fn solo_ids_de_la_ultima_busqueda() {
        let last = vec![update(GUID)];
        assert!(validate(&[GUID.into()], &last).is_ok());
        assert!(validate(&["00000000-0000-0000-0000-000000000000".into()], &last).is_err());
        assert!(validate(&[], &last).is_err());
    }

    #[test]
    fn instalar_exige_administrador() {
        let state = State::default();
        *state.last.lock().unwrap() = vec![update(GUID)];
        assert!(install(&[GUID.into()], false, &state).is_err());
    }

    #[test]
    fn fabricantes_de_la_lista() {
        assert!(vendor_url("nvidia").unwrap().starts_with("https://www.nvidia.com/"));
        assert!(vendor_url("https://malware.example").is_none());
        assert!(open_vendor_page("otro").is_err());
    }

    /// Solo lectura: `cargo test -- --ignored --nocapture`.
    #[test]
    #[ignore]
    fn inventario_y_busqueda_reales() {
        let drivers = list().unwrap();
        println!("{} controladores", drivers.len());
        for d in drivers.iter().filter(|d| d.class.as_deref() == Some("DISPLAY")) {
            println!("{} | {:?} | {:?} | {:?}", d.device, d.provider, d.version, d.date);
        }
        let updates = search(&State::default()).unwrap();
        println!("{} actualizaciones", updates.len());
        for u in &updates {
            println!("{} | {:?} | {:?}", u.title, u.class, u.date);
        }
    }
}
