# Design

## Context

Ver `proposal.md`. En el equipo de desarrollo hay 7 tareas de terceros con disparador de inicio de sesión y unos 25 servicios automáticos fuera de `C:\Windows`, entre ellos Microsoft Defender (`WinDefend`, `MDCoreSvc`), instalado en `%ProgramData%\Microsoft\Windows Defender`.

## Goals / Non-Goals

**Goals:**
- Reutilizar `Source`, `StartupEntry`, el log y History.

**Non-Goals:**
- Arrancar o parar servicios en caliente.

## Decisions

### Nuevos orígenes

`Source::ScheduledTask` ("Tarea programada") y `Source::Service` ("Servicio"), ambos con administrador. Su nombre interno es la ruta completa de la tarea (`\EasyTune`) o el nombre del servicio.

### Listado con una llamada a PowerShell

`Get-ScheduledTask` (tareas fuera de `\Microsoft\` con `MSFT_TaskLogonTrigger` o `MSFT_TaskBootTrigger`) y `Get-CimInstance Win32_Service` (automáticos o en `services.json`), en un solo script que devuelve JSON. En Rust se descartan los de sistema: ejecutable (con variables expandidas) dentro de `%SystemRoot%` o de una carpeta `Windows Defender`, y una lista fija de servicios de seguridad (`WinDefend`, `MDCoreSvc`, `WdNisSvc`, `Sense`, `SecurityHealthService`). El editor se resuelve igual que para el resto de entradas.

### Cambiar estado

- Tareas: `Enable-ScheduledTask` / `Disable-ScheduledTask` con `-TaskPath` y `-TaskName` escapados.
- Servicios: valor `Start` de `HKLM\SYSTEM\CurrentControlSet\Services\<nombre>` (2 automático, 3 manual). No se toca `DelayedAutostart`, así que el inicio retrasado se conserva. Surte efecto al reiniciar. Los nombres pasados a manual se guardan en `services.json` para seguir listándolos.

Antes de cambiar se vuelve a listar y se comprueba que la entrada existe y no es de sistema.

## Risks / Trade-offs

- [Un programa necesita su servicio al arrancar] → Manual, no deshabilitado: el sistema lo arranca bajo demanda; y se puede volver a activar desde la app o History.
- [Software de seguridad de terceros] → Suelen estar protegidos y el cambio falla con error visible; Defender se excluye explícitamente.
