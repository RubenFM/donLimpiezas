# Proposal

> **Estado: borrador.** Pendiente de revisar antes de generar specs, design y tasks.

## Why

Además de los programas de inicio, muchos servicios, tareas programadas y opciones de Windows (telemetría, sugerencias, anuncios) consumen recursos o privacidad sin aportar nada al usuario.

## What Changes

- Catálogo cerrado de ajustes reversibles, cada uno con descripción, impacto y valor original guardado:
  - Telemetría: servicio `DiagTrack`, `dmwappushservice`, tareas programadas de telemetría.
  - Sugerencias y anuncios del menú Inicio, pantalla de bloqueo y Explorador; ID de publicidad.
  - Widgets, Copilot, búsqueda web en el menú Inicio.
  - Plan de energía de alto rendimiento (opcional).
- "Modo reposo" para apps en segundo plano: listar procesos de usuario que consumen recursos y permitir cerrarlos.
- Botón "Restaurar valores originales" por ajuste y global.

## Capabilities

### New Capabilities
- `system-tweaks`: ajustes de rendimiento y privacidad reversibles desde un catálogo revisado.

### Modified Capabilities

## Impact

- Nuevo `src-tauri/src/tweaks.rs`; escribe en registro (`winreg`), servicios y tareas programadas; la mayoría requiere administrador.
- Cada cambio se guarda con su valor anterior en `%LOCALAPPDATA%\<app>\tweaks.json` para poder revertirlo.

## Open Questions

- ¿Incluir "modo reposo" de procesos aquí o en un cambio aparte?
