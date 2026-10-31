# Design

## Context

Ver `proposal.md`. Todos los ajustes del catálogo son valores `REG_DWORD` en `HKCU` o `HKLM` (incluido el tipo de inicio de servicios, `HKLM\SYSTEM\CurrentControlSet\Services\<servicio>\Start`). `winreg` ya está en el proyecto.

## Goals / Non-Goals

**Goals:**
- Un único mecanismo (valores DWORD) para todos los ajustes, fácil de revisar y de deshacer.

**Non-Goals:**
- Parar servicios en caliente: el cambio de tipo de inicio surte efecto al reiniciar y así se indica.

## Decisions

### Modelo

```rust
struct Setting { root: Root, path: &str, name: &str, value: u32 }
struct Tweak { id, name, description, group, note: Option<&str>, settings: &[Setting] }
```

Un ajuste requiere administrador si alguno de sus valores está en `HKLM`. Está aplicado si todos sus valores existen y coinciden.

### Valores originales

`tweaks.json` guarda `{ id: [{ root, path, name, value: Option<u32> }] }`. Al aplicar solo se guarda si no hay entrada para ese id (preserva el primer original). Al restaurar se escribe cada valor o se borra si era `None`, y se elimina la entrada. Si el archivo no existe o está corrupto se trata como vacío: en el peor caso no se puede restaurar, nunca se restaura algo incorrecto.

Las funciones de aplicar y restaurar reciben el ajuste y el almacén como parámetros, para poder probarlas con un ajuste de prueba en `HKCU\Software\DonLimpiezasTest`.

### Catálogo inicial

| id | Grupo | Admin | Valores |
|---|---|---|---|
| `telemetry_policy` | Telemetría | Sí | `HKLM\SOFTWARE\Policies\Microsoft\Windows\DataCollection\AllowTelemetry = 0` |
| `diagtrack_service` | Telemetría | Sí | `Services\DiagTrack\Start = 4` (reinicio) |
| `dmwappush_service` | Telemetría | Sí | `Services\dmwappushservice\Start = 4` (reinicio) |
| `advertising_id` | Telemetría | No | `AdvertisingInfo\Enabled = 0` |
| `tailored_experiences` | Telemetría | No | `Privacy\TailoredExperiencesWithDiagnosticDataEnabled = 0` |
| `start_suggestions` | Anuncios | No | `ContentDeliveryManager\SubscribedContent-338388Enabled = 0` |
| `lockscreen_tips` | Anuncios | No | `RotatingLockScreenOverlayEnabled = 0`, `SubscribedContent-338387Enabled = 0` |
| `settings_suggestions` | Anuncios | No | `SubscribedContent-338393/353694/353696Enabled = 0` |
| `tips_notifications` | Anuncios | No | `SubscribedContent-338389Enabled = 0` |
| `silent_app_installs` | Anuncios | No | `SilentInstalledAppsEnabled = 0` |
| `explorer_sync_ads` | Anuncios | No | `Explorer\Advanced\ShowSyncProviderNotifications = 0` |
| `widgets` | Windows 11 | Sí | `HKLM\SOFTWARE\Policies\Microsoft\Dsh\AllowNewsAndInterests = 0` |
| `copilot` | Windows 11 | No | `HKCU\Software\Policies\Microsoft\Windows\WindowsCopilot\TurnOffWindowsCopilot = 1` |
| `start_web_search` | Windows 11 | No | `HKCU\Software\Policies\Microsoft\Windows\Explorer\DisableSearchBoxSuggestions = 1` |
| `game_dvr` | Juegos | No | `HKCU\System\GameConfigStore\GameDVR_Enabled = 0` |

### Interfaz

Lista agrupada. Los ajustes no aplicados tienen casilla; los aplicados muestran "Aplicado" y, si hay original guardado, un botón "Restaurar". Barra con "Aplicar seleccionados" y "Restaurar todo".

## Risks / Trade-offs

- [Windows ignora o reescribe alguna clave en futuras versiones] → El estado se lee siempre del registro real; el usuario ve si sigue aplicado.
- [`AllowTelemetry = 0` solo tiene efecto completo en Enterprise/Education] → Se indica en la nota del ajuste.
- [Desactivar DiagTrack afecta a diagnósticos que pide soporte de Microsoft] → Se puede restaurar en cualquier momento.
