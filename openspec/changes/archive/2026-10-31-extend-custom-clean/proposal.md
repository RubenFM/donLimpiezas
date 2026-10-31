# Proposal

## Why

Custom Clean cubre lo básico de Windows y los navegadores. Faltan cachés habituales de Windows y de aplicaciones de uso diario, y opciones de privacidad que cualquier limpiador ofrece. Además, limpiar la caché de un programa abierto deja archivos sin borrar sin que el usuario sepa por qué.

## What Changes

- Nuevas categorías de **Windows**: caché de iconos, Prefetch (admin) y logs de mantenimiento de Windows (CBS y DISM, admin).
- Nuevo grupo **Aplicaciones**: cachés de Discord, Spotify, Visual Studio Code y del navegador integrado de Steam.
- Nuevo grupo **Privacidad**, **desmarcado por defecto** y con aviso: historial y cookies de Chrome, Edge y Brave; cookies de Firefox; documentos recientes y listas de accesos rápidos.
- Aviso **"Abierto"** en las categorías cuyo programa se está ejecutando, indicando que conviene cerrarlo para limpiarlo por completo.
- Fuera de alcance: exclusiones definidas por el usuario, historial de Firefox (comparte base de datos con los marcadores) e historial de Ejecutar (registro). Pueden ser cambios posteriores.

## Capabilities

### New Capabilities

### Modified Capabilities
- `system-cleanup`: nuevas categorías y grupos, categorías desmarcadas por defecto y aviso de programa abierto.

## Impact

- `src-tauri/src/cleaner.rs`: nuevos campos de categoría (`default_selected`, `warning`, `processes`), destino `Files` para archivos concretos y detección de procesos.
- `windows-sys`: feature `Win32_System_Diagnostics_ToolHelp`.
- `src/modules/custom-clean.ts`: selección por defecto, aviso y etiqueta "Abierto".
