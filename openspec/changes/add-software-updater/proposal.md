# Proposal

> **Estado: borrador.** Pendiente de revisar antes de generar specs, design y tasks.

## Why

Los programas desactualizados son la vía más común de vulnerabilidades, y actualizarlos uno a uno es tedioso.

## What Changes

- Detectar actualizaciones disponibles mediante **winget** (`winget upgrade`), que ya viene con Windows 10/11 y usa instaladores oficiales.
- Listar programa, versión instalada y disponible; actualizar seleccionados o todos con progreso por programa.
- Indicador de avisos en la navegación cuando hay actualizaciones.

## Capabilities

### New Capabilities
- `software-updates`: detección y aplicación de actualizaciones de programas mediante winget.

### Modified Capabilities

## Impact

- Nuevo `src-tauri/src/updater.rs`; depende de que `winget` esté presente (avisar si no).
- La salida de `winget` es texto; valorar `winget export`/módulo `Microsoft.WinGet.Client` de PowerShell para obtener datos estructurados.
