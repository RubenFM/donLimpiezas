# Proposal

> **Estado: borrador.** Pendiente de revisar antes de generar specs, design y tasks.

## Why

Fotos, descargas y copias de seguridad duplicadas ocupan mucho espacio y son difíciles de localizar a mano.

## What Changes

- El usuario elige carpetas (por defecto: Documentos, Descargas, Imágenes, Vídeos, Música) y filtros (tamaño mínimo, extensiones).
- Detección por tamaño y después por hash de contenido (no por nombre), en segundo plano con progreso y cancelación.
- Resultados agrupados; selección automática "conservar el más antiguo/reciente" y manual.
- Los archivos se envían a la **papelera**, nunca se borran directamente.

## Capabilities

### New Capabilities
- `duplicate-files`: búsqueda de archivos duplicados por contenido y eliminación a la papelera.

### Modified Capabilities

## Impact

- Nuevo `src-tauri/src/duplicates.rs`; crate de hash rápido (p. ej. `blake3`) y eventos de Tauri para el progreso.
- Excluir carpetas del sistema y de OneDrive sin descargar (marcadores de posición en la nube).
