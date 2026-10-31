# Proposal

## Why

Windows guarda versiones antiguas de sus componentes tras cada actualización en la carpeta WinSxS. Limpiarla con DISM suele liberar varios GB, pero hay que saber el comando y ejecutarlo como administrador.

## What Changes

- Nueva categoría de Custom Clean, **"Componentes antiguos de Windows Update"**: ejecuta la limpieza oficial de Windows (`DISM /Online /Cleanup-Image /StartComponentCleanup`).
- Requiere administrador, va **desmarcada por defecto** y avisa de que tarda varios minutos.
- Su tamaño no se conoce antes de limpiar (calcularlo tarda tanto como limpiar): se muestra como desconocido y, al terminar, se informa del espacio liberado midiendo el espacio libre del disco del sistema.
- Fuera de alcance: `/ResetBase` (impide desinstalar actualizaciones) y análisis previo del almacén.

## Capabilities

### New Capabilities

### Modified Capabilities
- `system-cleanup`: categoría de limpieza del almacén de componentes.

## Impact

- `src-tauri/src/cleaner.rs`: destino `ComponentStore` y campo `sizeKnown` en el análisis.
- `src-tauri/src/sysinfo.rs`: espacio libre de un disco reutilizable.
- `src/modules/custom-clean.ts`: tamaño desconocido.
