# Proposal

> **Estado: borrador.** Pendiente de revisar antes de generar specs, design y tasks.

## Why

Con nueve módulos, las pestañas superiores no escalan. Hace falta una navegación lateral fija que sirva de esqueleto para el resto de cambios.

## What Changes

- Sustituir las pestañas por una barra lateral con icono y nombre de cada módulo, en este orden: Revisión, Limpieza, Desinstalador, Apps preinstaladas, Duplicados, Nubes, Rendimiento, Inicio, Actualizar software, Controladores.
- Indicador (punto) en los módulos con avisos pendientes, alimentado por cada módulo.
- Recordar el último módulo abierto.
- Iconos propios (SVG en línea), no copiados de otra app.

## Capabilities

### New Capabilities
- `app-navigation`: estructura de navegación entre módulos y avisos por módulo.

### Modified Capabilities

## Impact

- `index.html`, `src/main.ts` (convendría dividirlo en un archivo por módulo en `src/modules/`), `src/styles.css`.
- Debería aplicarse **antes** que el resto de cambios con interfaz.

## Open Questions

- ¿Nombres de los módulos en español o en inglés?
