# Proposal

## Why

Los programas desactualizados son la vía más común de vulnerabilidades, y actualizarlos uno a uno es tedioso. Windows 10/11 ya incluye winget, que actualiza con los instaladores oficiales de cada programa.

## What Changes

- Nuevo módulo **Software Updater** en la barra lateral, tras Startup Manager.
- Lista de programas con actualización disponible según winget (incluidos los de versión desconocida), con versión instalada y disponible.
- Actualizar los seleccionados de uno en uno con el instalador oficial (winget pide elevación si hace falta), con progreso y resultado por programa.
- **Contador en la barra lateral**: al abrir la app se comprueban las actualizaciones en segundo plano y el módulo muestra cuántas hay.
- Registro en `%LOCALAPPDATA%\DonLimpiezas\updates.log`.
- Fuera de alcance: actualizaciones de Windows y de la Microsoft Store, y fijar o ignorar versiones.

## Capabilities

### New Capabilities
- `software-updates`: detección y aplicación de actualizaciones de programas mediante winget.

### Modified Capabilities
- `app-navigation`: los módulos pueden mostrar un contador de avisos en la barra lateral y comprobar datos al arrancar la app.

## Impact

- Nuevo `src-tauri/src/updater.rs` y comandos `list_updates` y `update_program`.
- Contrato de módulo: `init()` opcional al arrancar; nuevo `src/nav.ts` para contadores.
- Nuevo `src/modules/software-updater.ts` y caché compartida `src/updates.ts`.
