# Proposal

## Why

Los programas que se inician con Windows son una de las principales causas de arranques lentos y consumo de memoria en segundo plano, y el usuario rara vez sabe cuáles tiene. Es la funcionalidad más usada de los limpiadores tras la limpieza de archivos, y encaja con el principio de Don Limpiezas de hacer cambios reversibles.

## What Changes

- Nueva pestaña **"Inicio"** que lista los programas que se ejecutan al iniciar sesión, con nombre, comando, editor (cuando se pueda obtener), origen y estado (activado/desactivado).
- Orígenes cubiertos: claves `Run` del registro (usuario y equipo, 64 y 32 bits) y carpetas de Inicio (usuario y comunes).
- Activar y desactivar entradas usando el mismo mecanismo que el Administrador de tareas (`StartupApproved`), de modo que los cambios son reversibles y coherentes con lo que muestra Windows.
- Las entradas de equipo (HKLM y carpeta de Inicio común) requieren administrador y se muestran deshabilitadas sin él.
- Se registra cada cambio en `%LOCALAPPDATA%\DonLimpiezas\startup.log`.
- Fuera de alcance: borrar entradas, tareas programadas, servicios, `RunOnce` y medir el impacto en el arranque. Pueden ser cambios posteriores.

## Capabilities

### New Capabilities
- `startup-programs`: detección de programas de inicio de sesión y su activación/desactivación reversible.

### Modified Capabilities

## Impact

- **Backend**: nuevo módulo `src-tauri/src/startup.rs` y comandos `list_startup` y `set_startup_enabled` en `lib.rs`.
- **Dependencias**: crates `winreg` (acceso al registro) y `chrono` (fecha del log). Ver `design.md`.
- **Frontend**: nueva pestaña en `index.html` y su lógica en `src/main.ts`.
- **Sistema**: escribe valores en `HKCU`/`HKLM\Software\Microsoft\Windows\CurrentVersion\Explorer\StartupApproved\*`. No modifica ni borra las entradas originales.
