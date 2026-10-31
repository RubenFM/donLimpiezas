# Proposal

## Why

Fotos, descargas y copias de seguridad duplicadas ocupan mucho espacio y son difíciles de localizar a mano, sobre todo cuando tienen nombres distintos.

## What Changes

- Nuevo módulo **Duplicate Finder** en la barra lateral, entre Bloatware Remover y Performance Optimizer.
- Carpetas por defecto: Documentos, Descargas, Imágenes, Vídeos y Música del usuario (respetando su ubicación real, p. ej. en OneDrive). Se pueden añadir y quitar carpetas.
- Tamaño mínimo configurable (por defecto 1 MB).
- Detección por contenido: agrupa por tamaño y confirma con hash, en segundo plano, con progreso y cancelación.
- Resultados agrupados ordenados por espacio recuperable; selección automática "conservar el más antiguo" y selección manual.
- Los archivos se envían a la **papelera**. Nunca se permite eliminar todas las copias de un grupo.
- Fuera de alcance: filtros por extensión, comparación de imágenes similares y búsqueda en unidades completas.

## Capabilities

### New Capabilities
- `duplicate-files`: búsqueda de archivos duplicados por contenido y eliminación segura a la papelera.

### Modified Capabilities

## Impact

- Nuevo `src-tauri/src/duplicates.rs`, comandos `default_duplicate_folders`, `find_duplicates`, `cancel_duplicates` y `trash_duplicates`, y evento de progreso.
- Dependencias: `blake3` (hash) y `tauri-plugin-dialog` (+ `@tauri-apps/plugin-dialog`) para elegir carpetas.
- Nuevo `src/modules/duplicate-finder.ts`.
