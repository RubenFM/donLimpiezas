# Tasks

## 1. Dependencias

- [x] 1.1 Añadir `blake3` y `tauri-plugin-dialog` (Rust), `@tauri-apps/plugin-dialog` (npm), registrar el plugin y el permiso `dialog:allow-open`; verificar que `cargo build` y `npx tsc --noEmit` pasan

## 2. Backend

- [x] 2.1 Implementar el recorrido, la agrupación por tamaño y los hashes parcial y completo en `duplicates.rs`; verificar con un test en una carpeta temporal (duplicados con distinto nombre, mismo tamaño con distinto contenido, archivos bajo el mínimo)
- [x] 2.2 Implementar progreso por eventos y cancelación; verificar con un test que la cancelación devuelve error
- [x] 2.3 Implementar `trash_duplicates` con validación (rutas desconocidas, grupo completo) y papelera; verificar con tests que rechaza ambos casos sin borrar nada
- [x] 2.4 Implementar las carpetas por defecto y exponer los comandos; verificar con un test `#[ignore]` que devuelve carpetas existentes

## 3. Interfaz

- [x] 3.1 Crear `src/modules/duplicate-finder.ts` (carpetas, mínimo, progreso, cancelar, grupos, selección automática, papelera) y registrarlo entre Bloatware Remover y Performance Optimizer; verificar `npx tsc --noEmit`, `npx vite build` y una captura con datos simulados

## 4. Integración y documentación

- [x] 4.1 Analizar Descargas en la app real, enviar un duplicado a la papelera y restaurarlo desde ella
- [x] 4.2 Actualizar `README.md`; verificar que coincide con el código

## Workflow follow-up

- Probar en la app real y archivar con `/opsx:archive`.
