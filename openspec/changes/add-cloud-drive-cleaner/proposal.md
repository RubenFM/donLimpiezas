# Proposal

> **Estado: borrador.** Pendiente de revisar antes de generar specs, design y tasks.

## Why

OneDrive, Google Drive y Dropbox acumulan archivos grandes y antiguos que ocupan espacio en la nube y, si están sincronizados localmente, también en disco.

## What Changes

- **Fase 1 (local, sin credenciales):** detectar las carpetas sincronizadas de OneDrive, Google Drive y Dropbox; listar archivos grandes, antiguos y duplicados; liberar espacio local marcándolos como "solo en la nube" (OneDrive: atributo de Archivos a petición) sin borrarlos.
- **Fase 2 (opcional, posterior):** conexión mediante las APIs oficiales (OAuth) para analizar la cuenta completa.

## Capabilities

### New Capabilities
- `cloud-storage-cleanup`: análisis de carpetas de nube sincronizadas y liberación de espacio local.

### Modified Capabilities

## Impact

- Nuevo `src-tauri/src/cloud.rs`. La fase 2 implicaría registrar apps OAuth en Microsoft, Google y Dropbox y guardar tokens de forma segura.
- Reutiliza la detección de duplicados de `add-duplicate-finder`.

## Open Questions

- ¿Basta con la fase 1 o la fase 2 es imprescindible?
