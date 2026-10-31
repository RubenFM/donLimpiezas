# Tasks

## 1. Backend

- [x] 1.1 Implementar en `history.rs` la lectura de todos los logs a entradas comunes; verificar con tests de cada formato y de líneas dañadas
- [x] 1.2 Añadir `Source::from_label`, `cloud::keep_local` y la columna de acción en `cloud.log`; verificar con tests
- [x] 1.3 Calcular la acción y su validez para cada entrada y ejecutar `undo_history(id)` recalculando desde el log; verificar con tests que no ofrece deshacer cuando la entrada no es la última
- [x] 1.4 Exponer `list_history` y `undo_history`; verificar que `cargo build` compila sin warnings y con un test `#[ignore]` que lee los logs reales

## 2. Interfaz

- [x] 2.1 Crear `src/modules/history.ts` y registrarlo el último; verificar `npx tsc --noEmit`, `npx vite build` y una captura con datos simulados

## 3. Integración y documentación

- [ ] 3.1 Restaurar desde el historial un ajuste aplicado en la app real y comprobarlo en Performance Optimizer
- [x] 3.2 Actualizar `README.md`; verificar que coincide con el código

## Workflow follow-up

- Probar en la app real y archivar con `/opsx:archive`.
