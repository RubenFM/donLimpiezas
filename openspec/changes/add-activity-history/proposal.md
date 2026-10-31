# Proposal

## Why

Cada módulo ya registra lo que cambia en su log de `%LOCALAPPDATA%\DonLimpiezas`, pero el usuario no lo ve y, para deshacer algo, tiene que recordar en qué módulo lo hizo. Una pantalla común con todo lo hecho y la forma de revertirlo da confianza para usar la app.

## What Changes

- Nuevo módulo **History**, el último de la barra lateral.
- Lista de todas las acciones registradas por los módulos, de la más reciente a la más antigua, agrupadas por día y filtrables por módulo, con su resultado.
- Acción de vuelta atrás cuando es posible y sigue siendo válida:
  - Programas de inicio: volver al estado anterior.
  - Ajustes de Performance Optimizer: restaurar o volver a aplicar.
  - Apps quitadas con Bloatware Remover: buscarla en Microsoft Store.
  - Duplicados: abrir la papelera de reciclaje.
  - Archivos liberados de la nube: volver a descargarlos y mantenerlos en el equipo.
- Desinstalaciones, actualizaciones de programas y controladores se muestran sin acción (no se pueden revertir desde la app).
- Fuera de alcance: borrar el historial y exportarlo.

## Capabilities

### New Capabilities
- `activity-history`: historial unificado de cambios y vuelta atrás.

### Modified Capabilities

## Impact

- Nuevo `src-tauri/src/history.rs` con comandos `list_history` y `undo_history`; reutiliza `startup::set_enabled`, `tweaks::apply/revert` y una nueva `cloud::keep_local`.
- `cloud.log` añade una columna de acción en las líneas nuevas (las antiguas se leen como "liberado").
- Nuevo `src/modules/history.ts`.
