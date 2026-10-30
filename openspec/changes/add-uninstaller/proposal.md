# Proposal

> **Estado: borrador.** Pendiente de revisar antes de generar specs, design y tasks.

## Why

Desinstalar desde Configuración de Windows es lento, no muestra tamaños ni fechas y no permite desinstalar varios programas seguidos.

## What Changes

- Listado de programas instalados (claves `Uninstall` de HKLM 64/32 bits y HKCU) y apps de la Store, con nombre, editor, versión, tamaño y fecha de instalación.
- Búsqueda, orden por tamaño/fecha/nombre.
- Desinstalar lanzando el desinstalador oficial de cada programa (`UninstallString` / `QuietUninstallString` si el usuario lo elige); cola para desinstalar varios.
- Fuera de alcance en este cambio: buscar restos tras desinstalar (posible cambio posterior).

## Capabilities

### New Capabilities
- `program-uninstall`: inventario de programas instalados y desinstalación mediante su desinstalador oficial.

### Modified Capabilities

## Impact

- Nuevo `src-tauri/src/uninstaller.rs`; reutiliza `winreg` (introducido en `add-startup-manager`).
- Relación con `bloatware-removal`: las apps AppX del catálogo siguen en su módulo; aquí se listan todas.

## Open Questions

- ¿Permitir desinstalación silenciosa o siempre con el asistente del programa?
