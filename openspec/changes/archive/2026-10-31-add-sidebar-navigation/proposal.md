# Proposal

## Why

La app va a tener diez módulos y las pestañas superiores no escalan. Hace falta una navegación lateral fija que sirva de esqueleto para el resto de cambios, y separar el código de cada módulo antes de que `src/main.ts` crezca más.

## What Changes

- Sustituir las pestañas por una barra lateral con icono y nombre de cada módulo. Los nombres van en inglés; el contenido de cada módulo sigue en español.
- Orden previsto: Health Check, Custom Clean, Uninstaller, Bloatware Remover, Duplicate Finder, Cloud Drive Cleaner, Performance Optimizer, Startup Manager, Software Updater, Driver Updater.
- Solo aparecen los módulos implementados. En este cambio: **Custom Clean** (la limpieza actual) y **Bloatware Remover** (las apps preinstaladas actuales). Cada cambio posterior añade su entrada.
- Recordar el último módulo abierto entre sesiones.
- Iconos SVG propios en línea.
- Reorganizar el frontend: un archivo por módulo en `src/modules/` y un registro común.
- Fuera de alcance: indicadores de avisos por módulo (llegarán con el primer módulo que los necesite, `add-software-updater`).

## Capabilities

### New Capabilities
- `app-navigation`: navegación entre módulos de la app y persistencia del módulo activo.

### Modified Capabilities

## Impact

- `index.html`, `src/main.ts` (pasa a ser solo el arranque), nuevos `src/modules/*.ts`, `src/styles.css`.
- Sin cambios en el backend ni en el comportamiento de limpieza y bloatware.
- Debe aplicarse **antes** que el resto de cambios con interfaz; `add-startup-manager` ya se adapta para añadirse como módulo.
