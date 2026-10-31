# Proposal

## Why

Windows trae activadas por defecto telemetría, anuncios y sugerencias que consumen recursos y datos sin aportar nada al usuario. Desactivarlas a mano exige conocer decenas de claves de registro, y deshacerlo es aún más difícil.

## What Changes

- Nuevo módulo **Performance Optimizer** en la barra lateral, entre Bloatware Remover y Startup Manager.
- Catálogo cerrado de ajustes agrupados (Telemetría, Anuncios y sugerencias, Windows 11 y juegos), cada uno con descripción, si requiere administrador y avisos (p. ej. "requiere reiniciar").
- Cada ajuste muestra si ya está aplicado. Aplicar varios a la vez con confirmación.
- Antes de aplicar se guarda el valor original de cada clave; **Restaurar** devuelve exactamente ese valor (o borra la clave si no existía), por ajuste o todos a la vez.
- Registro en `%LOCALAPPDATA%\DonLimpiezas\tweaks.log`.
- Fuera de alcance: tareas programadas, plan de energía y cierre de procesos en segundo plano ("modo reposo"). Pueden ser cambios posteriores.

## Capabilities

### New Capabilities
- `system-tweaks`: ajustes de privacidad y rendimiento reversibles desde un catálogo revisado.

### Modified Capabilities

## Impact

- Nuevo `src-tauri/src/tweaks.rs` y comandos `list_tweaks`, `apply_tweaks` y `revert_tweaks`.
- Valores originales en `%LOCALAPPDATA%\DonLimpiezas\tweaks.json`; `log.rs` expone la carpeta de datos.
- Nuevo `src/modules/performance-optimizer.ts`.
