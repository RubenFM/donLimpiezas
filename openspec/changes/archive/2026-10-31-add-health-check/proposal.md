# Proposal

## Why

Con siete módulos, el usuario no sabe por dónde empezar. Un resumen de solo lectura del estado del equipo, con acceso directo a cada módulo, es la puerta de entrada natural a la app.

## What Changes

- Nuevo módulo **Health Check**, el primero de la barra lateral (y por tanto el que se abre la primera vez).
- Datos del equipo: versión de Windows, tiempo encendido, memoria y espacio libre de cada disco.
- Apartados que reutilizan los análisis de solo lectura de otros módulos, cada uno con su estado y un botón **Revisar** que abre el módulo: espacio liberable (Custom Clean), bloatware presente, programas de inicio activados, actualizaciones pendientes y ajustes de privacidad sin aplicar.
- Nunca modifica nada desde aquí.
- Fuera de alcance: puntuación global, duplicados (su análisis es lento) e informes históricos.

## Capabilities

### New Capabilities
- `health-check`: resumen de solo lectura del estado del equipo con acceso a cada módulo.

### Modified Capabilities
- `app-navigation`: un módulo puede abrir otro.

## Impact

- Nuevo `src-tauri/src/sysinfo.rs` y comando `system_info` (features `Win32_System_SystemInformation` y `Win32_Storage_FileSystem` de `windows-sys`).
- Nuevo `src/modules/health-check.ts`; reutiliza `scan_cleanup`, `list_bloat`, `list_startup`, `list_tweaks` y la caché de `src/updates.ts`.
- Depende de `add-software-updater` (navegación y caché de actualizaciones).
