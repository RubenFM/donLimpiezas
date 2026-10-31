# Tasks

## 1. Backend

- [x] 1.1 Implementar la detección de carpetas (OneDrive en el registro, Dropbox en `info.json`); verificar con un test del lector de `info.json` y un test `#[ignore]` que encuentra OneDrive personal
- [x] 1.2 Implementar el análisis por atributos (sin abrir archivos) con filtros de tamaño y antigüedad; verificar con un test de la clasificación de atributos
- [x] 1.3 Implementar la liberación con `attrib -P +U`, validación de rutas y `cloud.log`; verificar con tests que rechaza rutas ajenas o no analizadas
- [x] 1.4 Exponer `cloud_roots`, `scan_cloud` y `free_cloud_files`; verificar que `cargo build` compila sin warnings

## 2. Interfaz

- [x] 2.1 Crear `src/modules/cloud-drive-cleaner.ts` y registrarlo tras Duplicate Finder; verificar `npx tsc --noEmit`, `npx vite build` y una captura con datos simulados

## 3. Integración y documentación

- [ ] 3.1 Liberar un archivo real de OneDrive y comprobar en el Explorador que pasa a "solo en la nube" y se descarga al abrirlo
- [x] 3.2 Actualizar `README.md`; verificar que coincide con el código

## Workflow follow-up

- Probar en la app real y archivar con `/opsx:archive`.
