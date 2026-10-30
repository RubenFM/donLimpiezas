# Proposal

> **Estado: borrador.** Pendiente de revisar antes de generar specs, design y tasks.

## Why

La limpieza actual cubre lo básico. Para estar a la altura de un limpiador completo faltan categorías habituales y opciones de privacidad.

## What Changes

- Nuevas categorías candidatas: Prefetch, logs de Windows (`*.log` de `%SystemRoot%\Logs`), caché de Delivery Optimization, caché de iconos, papelera por unidad, cachés de apps (Discord, Spotify, Teams, VS Code, Steam shader cache).
- Opciones de privacidad **desmarcadas por defecto** y con aviso: historial, cookies y sesiones de navegadores (cierran sesión en webs); documentos recientes; historial de Ejecutar.
- Avisar si un navegador está abierto antes de limpiar su caché.
- Lista de exclusiones definida por el usuario.

## Capabilities

### New Capabilities

### Modified Capabilities
- `system-cleanup`: nuevas categorías, categorías de privacidad, detección de navegador abierto y exclusiones.

## Impact

- `src-tauri/src/cleaner.rs` (probablemente dividido en `cleaner/` por grupos), `src/main.ts`.
- Leer historial/cookies requiere tocar bases SQLite de los navegadores: valorar crate `rusqlite` o limitarse a borrar archivos completos.

## Open Questions

- ¿Qué apps de terceros priorizar en cachés de aplicaciones?
