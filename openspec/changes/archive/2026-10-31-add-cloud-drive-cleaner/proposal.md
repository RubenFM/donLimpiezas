# Proposal

## Why

OneDrive y Dropbox descargan copias locales de los archivos que se abren y no las liberan solos. Archivos grandes o antiguos que ya están a salvo en la nube acaban ocupando espacio en el disco sin que el usuario lo sepa.

## What Changes

- Nuevo módulo **Cloud Drive Cleaner** en la barra lateral, tras Duplicate Finder.
- Detecta las carpetas sincronizadas de OneDrive (todas las cuentas) y Dropbox.
- Lista los archivos descargados en local por encima de un tamaño mínimo y, opcionalmente, sin modificar desde hace un tiempo, con el espacio local total y el que se puede liberar.
- **Liberar espacio**: marca los archivos seleccionados como "solo en la nube", igual que la opción del Explorador. No borra nada: siguen en la nube y se descargan al abrirlos.
- Registro en `%LOCALAPPDATA%\DonLimpiezas\cloud.log`.
- Fuera de alcance: Google Drive (unidad virtual en streaming), conexión a las APIs de los servicios (OAuth) y duplicados en la nube (Duplicate Finder ya analiza OneDrive).

## Capabilities

### New Capabilities
- `cloud-storage-cleanup`: liberación del espacio local que ocupan archivos ya guardados en la nube.

### Modified Capabilities

## Impact

- Nuevo `src-tauri/src/cloud.rs` y comandos `cloud_roots`, `scan_cloud` y `free_cloud_files`.
- Nuevo `src/modules/cloud-drive-cleaner.ts`.
