# Tasks

## 1. Backend

- [ ] 1.1 Mover el punto de restauración a `system.rs` con descripción y adaptar Bloatware Remover; verificar que `cargo test` pasa
- [ ] 1.2 Implementar el inventario en `drivers.rs`; verificar con un test `#[ignore]` que devuelve la tarjeta gráfica real
- [ ] 1.3 Implementar la búsqueda y la instalación con validación de ids (GUID y última búsqueda) y `drivers.log`; verificar con tests de validación y un test `#[ignore]` de búsqueda real
- [ ] 1.4 Implementar `open_vendor_page` con la lista fija y exponer los comandos; verificar con un test que rechaza fabricantes desconocidos y que `cargo build` compila sin warnings

## 2. Interfaz

- [ ] 2.1 Crear `src/modules/driver-updater.ts` (actualizaciones, gráficas, inventario con filtro) y registrarlo el último; verificar `npx tsc --noEmit`, `npx vite build` y una captura con datos simulados

## 3. Integración y documentación

- [ ] 3.1 Buscar actualizaciones en la app real y abrir la página de NVIDIA desde el botón
- [ ] 3.2 Actualizar `README.md`; verificar que coincide con el código

## Workflow follow-up

- Probar en la app real y archivar con `/opsx:archive`.
