//! Detección y eliminación de aplicaciones preinstaladas (paquetes AppX).
//!
//! Solo se pueden quitar paquetes de `CATALOG`, una lista revisada de apps que
//! no afectan al funcionamiento de Windows. Nunca se tocan Microsoft Store, los
//! runtimes (VCLibs, .NET, UI.Xaml), el shell ni Seguridad de Windows.

use crate::ps;
use serde::{Deserialize, Serialize};

struct Entry {
    id: &'static str,
    name: &'static str,
    group: &'static str,
    note: &'static str,
}

const fn entry(id: &'static str, name: &'static str, group: &'static str, note: &'static str) -> Entry {
    Entry { id, name, group, note }
}

const MICROSOFT: &str = "Apps de Microsoft";
const GAMING: &str = "Xbox y juegos";
const PROMOS: &str = "Apps promocionadas de terceros";

const CATALOG: &[Entry] = &[
    entry("Microsoft.BingNews", "Noticias", MICROSOFT, ""),
    entry("Microsoft.BingWeather", "El Tiempo", MICROSOFT, ""),
    entry("Microsoft.GetHelp", "Obtener ayuda", MICROSOFT, ""),
    entry("Microsoft.Getstarted", "Recomendaciones / Introducción", MICROSOFT, ""),
    entry("Microsoft.MicrosoftOfficeHub", "Microsoft 365 (Office)", MICROSOFT, "Solo el lanzador; no desinstala Office."),
    entry("Microsoft.MicrosoftSolitaireCollection", "Microsoft Solitaire Collection", MICROSOFT, ""),
    entry("Microsoft.People", "Contactos", MICROSOFT, ""),
    entry("Microsoft.PowerAutomateDesktop", "Power Automate", MICROSOFT, ""),
    entry("Microsoft.Todos", "Microsoft To Do", MICROSOFT, ""),
    entry("Microsoft.WindowsFeedbackHub", "Centro de opiniones", MICROSOFT, ""),
    entry("Microsoft.WindowsMaps", "Mapas", MICROSOFT, ""),
    entry("Microsoft.ZuneVideo", "Películas y TV", MICROSOFT, ""),
    entry("Microsoft.ZuneMusic", "Reproductor multimedia", MICROSOFT, "Es el reproductor de música/vídeo por defecto de Windows 11."),
    entry("Microsoft.YourPhone", "Enlace móvil", MICROSOFT, ""),
    entry("Microsoft.MixedReality.Portal", "Portal de realidad mixta", MICROSOFT, ""),
    entry("Microsoft.Microsoft3DViewer", "Visor 3D", MICROSOFT, ""),
    entry("Microsoft.MSPaint", "Paint 3D", MICROSOFT, "No es el Paint clásico."),
    entry("Microsoft.SkypeApp", "Skype", MICROSOFT, ""),
    entry("Microsoft.549981C3F5F10", "Cortana", MICROSOFT, ""),
    entry("Microsoft.Copilot", "Copilot", MICROSOFT, ""),
    entry("Microsoft.OutlookForWindows", "Outlook (nuevo)", MICROSOFT, "Si lo usas como correo, no lo quites."),
    entry("Microsoft.Windows.DevHome", "Dev Home", MICROSOFT, ""),
    entry("MicrosoftTeams", "Microsoft Teams (personal)", MICROSOFT, ""),
    entry("MSTeams", "Microsoft Teams", MICROSOFT, "Si lo usas para trabajar, no lo quites."),
    entry("Clipchamp.Clipchamp", "Clipchamp", MICROSOFT, ""),
    entry("Microsoft.GamingApp", "Xbox", GAMING, "Necesaria para juegos de Game Pass."),
    entry("Microsoft.XboxApp", "Xbox Console Companion", GAMING, ""),
    entry("Microsoft.XboxGamingOverlay", "Xbox Game Bar", GAMING, "Usada para grabar pantalla con Win+G."),
    entry("Microsoft.XboxSpeechToTextOverlay", "Xbox voz a texto", GAMING, ""),
    entry("king.com.CandyCrushSaga", "Candy Crush Saga", PROMOS, ""),
    entry("king.com.CandyCrushSodaSaga", "Candy Crush Soda Saga", PROMOS, ""),
    entry("SpotifyAB.SpotifyMusic", "Spotify", PROMOS, ""),
    entry("Disney.37853FC22B2CE", "Disney+", PROMOS, ""),
    entry("AmazonVideo.PrimeVideo", "Prime Video", PROMOS, ""),
    entry("BytedancePte.Ltd.TikTok", "TikTok", PROMOS, ""),
];

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BloatApp {
    id: &'static str,
    name: &'static str,
    group: &'static str,
    note: &'static str,
    /// Instalada para el usuario actual.
    installed: bool,
    /// Incluida en la imagen de Windows (se instalaría en usuarios nuevos).
    provisioned: bool,
}

#[derive(Serialize, Deserialize)]
pub struct RemoveResult {
    name: String,
    ok: bool,
    error: Option<String>,
}

fn names_from_json(json: &str) -> Result<Vec<String>, String> {
    if json.is_empty() {
        return Ok(Vec::new());
    }
    serde_json::from_str(json).map_err(|e| format!("Respuesta inesperada de PowerShell: {e}"))
}

pub fn list(include_provisioned: bool) -> Result<Vec<BloatApp>, String> {
    let installed = names_from_json(&ps::run(
        "ConvertTo-Json -Compress -InputObject @(Get-AppxPackage | Select-Object -ExpandProperty Name)",
    )?)?;
    // Listar paquetes aprovisionados requiere administrador.
    let provisioned = if include_provisioned {
        names_from_json(&ps::run(
            "ConvertTo-Json -Compress -InputObject @(Get-AppxProvisionedPackage -Online | Select-Object -ExpandProperty DisplayName)",
        )?)?
    } else {
        Vec::new()
    };

    Ok(CATALOG
        .iter()
        .map(|e| BloatApp {
            id: e.id,
            name: e.name,
            group: e.group,
            note: e.note,
            installed: installed.iter().any(|n| n.eq_ignore_ascii_case(e.id)),
            provisioned: provisioned.iter().any(|n| n.eq_ignore_ascii_case(e.id)),
        })
        .collect())
}

pub fn remove(ids: &[String], all_users: bool) -> Result<Vec<RemoveResult>, String> {
    // Solo se aceptan ids del catálogo; además así es seguro incrustarlos en el script.
    let names = ids
        .iter()
        .map(|id| {
            CATALOG
                .iter()
                .find(|e| e.id == id)
                .map(|e| ps::quote(e.id))
                .ok_or_else(|| format!("Paquete no permitido: {id}"))
        })
        .collect::<Result<Vec<_>, _>>()?;
    if names.is_empty() {
        return Ok(Vec::new());
    }

    let script = format!(
        r#"$ErrorActionPreference = 'Stop'
$names = @({names})
$allUsers = ${all_users}
$log = Join-Path $env:LOCALAPPDATA 'DonLimpiezas\debloat.log'
New-Item -ItemType Directory -Force -Path (Split-Path $log) | Out-Null
$results = foreach ($n in $names) {{
    try {{
        if ($allUsers) {{
            Get-AppxPackage -AllUsers -Name $n | Remove-AppxPackage -AllUsers
            Get-AppxProvisionedPackage -Online | Where-Object DisplayName -eq $n | Remove-AppxProvisionedPackage -Online | Out-Null
        }} else {{
            Get-AppxPackage -Name $n | Remove-AppxPackage
        }}
        $r = [pscustomobject]@{{ name = $n; ok = $true; error = $null }}
    }} catch {{
        $r = [pscustomobject]@{{ name = $n; ok = $false; error = $_.Exception.Message }}
    }}
    $status = if ($r.ok) {{ 'OK' }} else {{ 'ERROR ' + $r.error }}
    Add-Content -Path $log -Value ("{{0}}`t{{1}}`t{{2}}" -f (Get-Date -Format o), $n, $status)
    $r
}}
ConvertTo-Json -Compress -InputObject @($results)"#,
        names = names.join(","),
        all_users = all_users,
    );

    let out = ps::run(&script)?;
    serde_json::from_str(&out).map_err(|e| format!("Respuesta inesperada de PowerShell: {e}"))
}
