# Tasks

## 1. Backend

- [x] 1.1 Implementar el recorrido con tamaños por carpeta, top de archivos, nube a 0 bytes, carpetas ilegibles, progreso y cancelación en `diskusage.rs`; verificar con un test en una carpeta temporal
- [x] 1.2 Implementar `disk_children` y `open_in_explorer` con validación; verificar con tests que rechaza rutas no analizadas
- [x] 1.3 Exponer los comandos y el estado; verificar que `cargo build` compila sin warnings y con un test `#[ignore]` que analiza la carpeta del usuario midiendo el tiempo

## 2. Interfaz

- [x] 2.1 Crear `src/modules/disk-analyzer.ts` (selector, progreso, migas, barras) y registrarlo antes de Duplicate Finder; verificar `npx tsc --noEmit`, `npx vite build` y una captura con datos simulados

## 3. Integración y documentación

- [ ] 3.1 Analizar C: en la app real y navegar hasta la carpeta del usuario
- [x] 3.2 Actualizar `README.md`; verificar que coincide con el código

## Workflow follow-up

- Probar en la app real y archivar con `/opsx:archive`.
