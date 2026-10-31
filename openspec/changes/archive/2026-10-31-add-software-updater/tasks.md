# Tasks

## 1. Backend

- [x] 1.1 Implementar el lector de la tabla de winget en `updater.rs`; verificar con tests de salidas en inglés y español, con indicador de progreso, versión desconocida y pie de tabla
- [x] 1.2 Implementar `list` y `update` con validación de ids, `updates.log` y error claro si falta winget; verificar con un test que rechaza ids desconocidos y con un test `#[ignore]` que lista las actualizaciones reales
- [x] 1.3 Exponer `list_updates` y `update_program`; verificar que `cargo build` compila sin warnings

## 2. Navegación

- [x] 2.1 Añadir `init?()` al contrato, `src/nav.ts` con `setBadge`/`navigate` y el contador en la barra lateral (también contraída); verificar `npx tsc --noEmit` y una captura

## 3. Interfaz

- [x] 3.1 Crear `src/updates.ts` y `src/modules/software-updater.ts` (lista, selección, cola con progreso) y registrarlo tras Startup Manager; verificar `npx vite build` y una captura con datos simulados

## 4. Integración y documentación

- [x] 4.1 Actualizar un programa real desde la app y comprobar que desaparece de la lista
- [x] 4.2 Actualizar `README.md`; verificar que coincide con el código

## Workflow follow-up

- Probar en la app real y archivar con `/opsx:archive`.
