//! Ajustes de privacidad y rendimiento reversibles.
//!
//! Todos son valores `REG_DWORD`. Antes de aplicar uno se guarda el valor
//! original de cada clave en `tweaks.json` para poder restaurarlo exactamente.

use crate::log;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use winreg::enums::{HKEY_CURRENT_USER, HKEY_LOCAL_MACHINE, KEY_READ, KEY_SET_VALUE};
use winreg::RegKey;

#[derive(Clone, Copy, PartialEq, Eq, Debug, Serialize, Deserialize)]
enum Root {
    Hkcu,
    Hklm,
}

impl Root {
    fn key(self) -> RegKey {
        RegKey::predef(match self {
            Root::Hkcu => HKEY_CURRENT_USER,
            Root::Hklm => HKEY_LOCAL_MACHINE,
        })
    }
}

struct Setting {
    root: Root,
    path: &'static str,
    name: &'static str,
    value: u32,
}

const fn cu(path: &'static str, name: &'static str, value: u32) -> Setting {
    Setting { root: Root::Hkcu, path, name, value }
}

const fn lm(path: &'static str, name: &'static str, value: u32) -> Setting {
    Setting { root: Root::Hklm, path, name, value }
}

struct Tweak {
    id: &'static str,
    name: &'static str,
    description: &'static str,
    group: &'static str,
    note: Option<&'static str>,
    settings: &'static [Setting],
}

impl Tweak {
    fn admin(&self) -> bool {
        self.settings.iter().any(|s| s.root == Root::Hklm)
    }
}

const TELEMETRY: &str = "Telemetría";
const ADS: &str = "Anuncios y sugerencias";
const WIN11: &str = "Windows 11";
const GAMES: &str = "Juegos";

const CDM: &str = r"Software\Microsoft\Windows\CurrentVersion\ContentDeliveryManager";
const RESTART: Option<&str> = Some("Surte efecto al reiniciar.");

const CATALOG: &[Tweak] = &[
    Tweak {
        id: "telemetry_policy",
        name: "Telemetría de Windows al mínimo",
        description: "Reduce los datos de diagnóstico que Windows envía a Microsoft.",
        group: TELEMETRY,
        note: Some("En Windows Home y Pro se queda en el nivel mínimo obligatorio."),
        settings: &[lm(r"SOFTWARE\Policies\Microsoft\Windows\DataCollection", "AllowTelemetry", 0)],
    },
    Tweak {
        id: "diagtrack_service",
        name: "Servicio de telemetría (DiagTrack)",
        description: "Desactiva el servicio «Experiencias del usuario y telemetría asociadas».",
        group: TELEMETRY,
        note: RESTART,
        settings: &[lm(r"SYSTEM\CurrentControlSet\Services\DiagTrack", "Start", 4)],
    },
    Tweak {
        id: "dmwappush_service",
        name: "Servicio de enrutamiento WAP Push",
        description: "Desactiva el servicio auxiliar que envía datos de telemetría.",
        group: TELEMETRY,
        note: RESTART,
        settings: &[lm(r"SYSTEM\CurrentControlSet\Services\dmwappushservice", "Start", 4)],
    },
    Tweak {
        id: "advertising_id",
        name: "ID de publicidad",
        description: "Impide que las apps usen tu ID de publicidad para mostrarte anuncios personalizados.",
        group: TELEMETRY,
        note: None,
        settings: &[cu(r"Software\Microsoft\Windows\CurrentVersion\AdvertisingInfo", "Enabled", 0)],
    },
    Tweak {
        id: "tailored_experiences",
        name: "Experiencias personalizadas",
        description: "Microsoft deja de usar tus datos de diagnóstico para sugerencias y anuncios.",
        group: TELEMETRY,
        note: None,
        settings: &[cu(
            r"Software\Microsoft\Windows\CurrentVersion\Privacy",
            "TailoredExperiencesWithDiagnosticDataEnabled",
            0,
        )],
    },
    Tweak {
        id: "start_suggestions",
        name: "Sugerencias en el menú Inicio",
        description: "Quita las apps recomendadas y promocionadas del menú Inicio.",
        group: ADS,
        note: None,
        settings: &[cu(CDM, "SubscribedContent-338388Enabled", 0)],
    },
    Tweak {
        id: "lockscreen_tips",
        name: "Datos curiosos en la pantalla de bloqueo",
        description: "Quita los consejos, trucos y anuncios de la pantalla de bloqueo.",
        group: ADS,
        note: None,
        settings: &[
            cu(CDM, "RotatingLockScreenOverlayEnabled", 0),
            cu(CDM, "SubscribedContent-338387Enabled", 0),
        ],
    },
    Tweak {
        id: "settings_suggestions",
        name: "Contenido sugerido en Configuración",
        description: "Quita las sugerencias que aparecen en la app Configuración.",
        group: ADS,
        note: None,
        settings: &[
            cu(CDM, "SubscribedContent-338393Enabled", 0),
            cu(CDM, "SubscribedContent-353694Enabled", 0),
            cu(CDM, "SubscribedContent-353696Enabled", 0),
        ],
    },
    Tweak {
        id: "tips_notifications",
        name: "Consejos y trucos de Windows",
        description: "Deja de mostrar notificaciones con sugerencias al usar Windows.",
        group: ADS,
        note: None,
        settings: &[cu(CDM, "SubscribedContent-338389Enabled", 0)],
    },
    Tweak {
        id: "silent_app_installs",
        name: "Instalación automática de apps promocionadas",
        description: "Evita que Windows instale por su cuenta apps como Candy Crush o Spotify.",
        group: ADS,
        note: None,
        settings: &[cu(CDM, "SilentInstalledAppsEnabled", 0)],
    },
    Tweak {
        id: "explorer_sync_ads",
        name: "Anuncios en el Explorador",
        description: "Quita las notificaciones de OneDrive y Microsoft 365 en el Explorador de archivos.",
        group: ADS,
        note: None,
        settings: &[cu(
            r"Software\Microsoft\Windows\CurrentVersion\Explorer\Advanced",
            "ShowSyncProviderNotifications",
            0,
        )],
    },
    Tweak {
        id: "widgets",
        name: "Widgets y noticias",
        description: "Desactiva el panel de Widgets y las noticias de la barra de tareas.",
        group: WIN11,
        note: Some("Puede requerir cerrar sesión."),
        settings: &[lm(r"SOFTWARE\Policies\Microsoft\Dsh", "AllowNewsAndInterests", 0)],
    },
    Tweak {
        id: "copilot",
        name: "Copilot en Windows",
        description: "Desactiva el asistente Copilot integrado en Windows.",
        group: WIN11,
        note: Some("En las versiones más recientes Copilot es una app aparte: quítala en Bloatware Remover."),
        settings: &[cu(r"Software\Policies\Microsoft\Windows\WindowsCopilot", "TurnOffWindowsCopilot", 1)],
    },
    Tweak {
        id: "start_web_search",
        name: "Búsqueda web en el menú Inicio",
        description: "La búsqueda del menú Inicio solo muestra resultados de tu equipo, sin Bing.",
        group: WIN11,
        note: Some("Surte efecto al reiniciar el Explorador o cerrar sesión."),
        settings: &[cu(r"Software\Policies\Microsoft\Windows\Explorer", "DisableSearchBoxSuggestions", 1)],
    },
    Tweak {
        id: "game_dvr",
        name: "Grabación en segundo plano de juegos",
        description: "Desactiva Game DVR, que graba continuamente mientras juegas y consume rendimiento.",
        group: GAMES,
        note: Some("Win+G sigue funcionando para capturas manuales."),
        settings: &[cu(r"System\GameConfigStore", "GameDVR_Enabled", 0)],
    },
];

#[derive(Serialize, Deserialize, Clone, PartialEq, Debug)]
struct Original {
    root: Root,
    path: String,
    name: String,
    /// `None` si la clave no existía.
    value: Option<u32>,
}

type Store = HashMap<String, Vec<Original>>;

fn store_path() -> PathBuf {
    log::data_dir().join("tweaks.json")
}

/// Si el archivo falta o está dañado se trata como vacío: en el peor caso no
/// se puede restaurar, pero nunca se restaura un valor incorrecto.
fn load(path: &Path) -> Store {
    fs::read_to_string(path)
        .ok()
        .and_then(|s| serde_json::from_str(&s).ok())
        .unwrap_or_default()
}

fn save(path: &Path, store: &Store) -> Result<(), String> {
    if let Some(dir) = path.parent() {
        fs::create_dir_all(dir).map_err(|e| e.to_string())?;
    }
    let json = serde_json::to_string_pretty(store).map_err(|e| e.to_string())?;
    fs::write(path, json).map_err(|e| format!("No se pudieron guardar los valores originales: {e}"))
}

fn read(root: Root, path: &str, name: &str) -> Option<u32> {
    root.key()
        .open_subkey_with_flags(path, KEY_READ)
        .ok()?
        .get_value::<u32, _>(name)
        .ok()
}

/// Escribe el valor, o lo borra si es `None` (sin error si ya no existía).
fn write(root: Root, path: &str, name: &str, value: Option<u32>) -> io::Result<()> {
    let ignore_missing = |r: io::Result<()>| match r {
        Err(e) if e.kind() == io::ErrorKind::NotFound => Ok(()),
        r => r,
    };
    match value {
        Some(v) => root.key().create_subkey(path)?.0.set_value(name, &v),
        None => match root.key().open_subkey_with_flags(path, KEY_SET_VALUE) {
            Ok(key) => ignore_missing(key.delete_value(name)),
            Err(e) => ignore_missing(Err(e)),
        },
    }
}

fn is_applied(tweak: &Tweak) -> bool {
    tweak.settings.iter().all(|s| read(s.root, s.path, s.name) == Some(s.value))
}

fn apply_one(tweak: &Tweak, store: &mut Store, store_path: &Path) -> Result<(), String> {
    // El original se guarda en disco antes de tocar el registro y solo la primera vez.
    if !store.contains_key(tweak.id) {
        let originals = tweak
            .settings
            .iter()
            .map(|s| Original {
                root: s.root,
                path: s.path.into(),
                name: s.name.into(),
                value: read(s.root, s.path, s.name),
            })
            .collect();
        store.insert(tweak.id.into(), originals);
        save(store_path, store)?;
    }
    for s in tweak.settings {
        write(s.root, s.path, s.name, Some(s.value))
            .map_err(|e| format!("No se pudo escribir {}: {e}", s.name))?;
    }
    Ok(())
}

fn revert_one(tweak: &Tweak, store: &mut Store, store_path: &Path) -> Result<(), String> {
    let originals = store
        .get(tweak.id)
        .ok_or("No hay valores originales guardados para este ajuste.")?;
    for o in originals {
        write(o.root, &o.path, &o.name, o.value).map_err(|e| format!("No se pudo restaurar {}: {e}", o.name))?;
    }
    store.remove(tweak.id);
    save(store_path, store)
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TweakState {
    id: &'static str,
    name: &'static str,
    description: &'static str,
    group: &'static str,
    note: Option<&'static str>,
    admin: bool,
    applied: bool,
    restorable: bool,
}

#[derive(Serialize)]
pub struct TweakResult {
    id: String,
    ok: bool,
    error: Option<String>,
}

pub fn list() -> Vec<TweakState> {
    let store = load(&store_path());
    CATALOG
        .iter()
        .map(|t| TweakState {
            id: t.id,
            name: t.name,
            description: t.description,
            group: t.group,
            note: t.note,
            admin: t.admin(),
            applied: is_applied(t),
            restorable: store.contains_key(t.id),
        })
        .collect()
}

/// Valida todos los ids antes de tocar nada.
fn resolve(ids: &[String], is_admin: bool) -> Result<Vec<&'static Tweak>, String> {
    ids.iter()
        .map(|id| {
            let tweak = CATALOG
                .iter()
                .find(|t| t.id == id)
                .ok_or_else(|| format!("Ajuste desconocido: {id}"))?;
            if tweak.admin() && !is_admin {
                return Err(format!("«{}» requiere administrador.", tweak.name));
            }
            Ok(tweak)
        })
        .collect()
}

fn run(
    ids: &[String],
    is_admin: bool,
    action: &str,
    f: fn(&Tweak, &mut Store, &Path) -> Result<(), String>,
) -> Result<Vec<TweakResult>, String> {
    let tweaks = resolve(ids, is_admin)?;
    let path = store_path();
    let mut store = load(&path);
    Ok(tweaks
        .into_iter()
        .map(|t| {
            let result = f(t, &mut store, &path);
            let state = match &result {
                Ok(()) => action.to_string(),
                Err(e) => format!("ERROR {e}"),
            };
            let _ = log::append("tweaks.log", &format!("{}\t{state}", t.id));
            TweakResult { id: t.id.into(), ok: result.is_ok(), error: result.err() }
        })
        .collect())
}

pub fn apply(ids: &[String], is_admin: bool) -> Result<Vec<TweakResult>, String> {
    run(ids, is_admin, "aplicado", apply_one)
}

pub fn revert(ids: &[String], is_admin: bool) -> Result<Vec<TweakResult>, String> {
    run(ids, is_admin, "restaurado", revert_one)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    const TEST_KEY: &str = r"Software\DonLimpiezasTest";

    #[test]
    fn ids_unicos() {
        let ids: HashSet<_> = CATALOG.iter().map(|t| t.id).collect();
        assert_eq!(ids.len(), CATALOG.len());
    }

    #[test]
    fn admin_si_toca_hklm() {
        let find = |id| CATALOG.iter().find(|t| t.id == id).unwrap();
        assert!(find("telemetry_policy").admin());
        assert!(find("diagtrack_service").admin());
        assert!(!find("advertising_id").admin());
        assert!(!find("lockscreen_tips").admin());
    }

    #[test]
    fn rechaza_ids_desconocidos_y_admin() {
        assert!(resolve(&["borrar-system32".into()], true).is_err());
        assert!(resolve(&["telemetry_policy".into()], false).is_err());
        assert!(resolve(&["advertising_id".into()], false).is_ok());
    }

    #[test]
    fn almacen_ida_y_vuelta() {
        let path = std::env::temp_dir().join(format!("donlimpiezas-tweaks-{}.json", std::process::id()));
        let mut store = Store::new();
        store.insert(
            "x".into(),
            vec![Original { root: Root::Hkcu, path: "A".into(), name: "B".into(), value: None }],
        );
        save(&path, &store).unwrap();
        assert_eq!(load(&path), store);
        fs::write(&path, "no es json").unwrap();
        assert!(load(&path).is_empty());
        fs::remove_file(&path).unwrap();
    }

    /// Usa una clave propia en HKCU que se borra al terminar.
    #[test]
    fn aplicar_y_restaurar_conserva_el_primer_original() {
        static SETTINGS: [Setting; 2] = [cu(TEST_KEY, "Nueva", 1), cu(TEST_KEY, "Existente", 0)];
        let tweak = Tweak {
            id: "test",
            name: "Test",
            description: "",
            group: "",
            note: None,
            settings: &SETTINGS,
        };
        let hkcu = Root::Hkcu.key();
        let _ = hkcu.delete_subkey_all(TEST_KEY);
        write(Root::Hkcu, TEST_KEY, "Existente", Some(5)).unwrap();
        let path = std::env::temp_dir().join(format!("donlimpiezas-tweaks-test-{}.json", std::process::id()));
        let mut store = Store::new();

        apply_one(&tweak, &mut store, &path).unwrap();
        assert!(is_applied(&tweak));
        assert_eq!(read(Root::Hkcu, TEST_KEY, "Nueva"), Some(1));
        assert_eq!(read(Root::Hkcu, TEST_KEY, "Existente"), Some(0));

        // Alguien lo cambia a mano y se vuelve a aplicar: el original no cambia.
        write(Root::Hkcu, TEST_KEY, "Existente", Some(7)).unwrap();
        apply_one(&tweak, &mut store, &path).unwrap();
        assert_eq!(load(&path)["test"][1].value, Some(5));

        revert_one(&tweak, &mut store, &path).unwrap();
        assert_eq!(read(Root::Hkcu, TEST_KEY, "Nueva"), None);
        assert_eq!(read(Root::Hkcu, TEST_KEY, "Existente"), Some(5));
        assert!(!is_applied(&tweak));
        assert!(load(&path).is_empty());
        assert!(revert_one(&tweak, &mut store, &path).is_err());

        hkcu.delete_subkey_all(TEST_KEY).unwrap();
        fs::remove_file(&path).unwrap();
    }
}
