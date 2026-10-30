# Proposal

> **Estado: borrador.** Pendiente de revisar antes de generar specs, design y tasks.

## Why

El usuario medio no sabe qué módulo usar. Un análisis en un clic que resuma el estado del equipo y proponga acciones es la puerta de entrada a la app.

## What Changes

- Pantalla "Revisión" con un botón "Analizar equipo" que ejecuta en modo solo lectura los análisis de otros módulos: espacio liberable (limpieza), bloatware presente, programas de inicio activados, actualizaciones de software pendientes.
- Resumen por apartado con botón "Solucionar", que lleva al módulo correspondiente con la selección hecha (nunca borra directamente desde aquí).
- Información básica del equipo: Windows, espacio libre por unidad, RAM, tiempo desde el último arranque.

## Capabilities

### New Capabilities
- `health-check`: análisis agregado de solo lectura y recomendaciones.

### Modified Capabilities

## Impact

- Depende de que existan los módulos que agrega; se puede implementar con los disponibles e ir añadiendo apartados.
- Nuevo módulo `src-tauri/src/health.rs` que reutiliza `cleaner::scan`, `debloat::list`, etc.

## Open Questions

- ¿Una "puntuación" global o solo la lista de apartados?
