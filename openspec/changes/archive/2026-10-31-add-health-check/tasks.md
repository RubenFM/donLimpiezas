# Tasks

## 1. Backend

- [x] 1.1 Implementar `sysinfo.rs` (Windows, tiempo encendido, memoria, discos fijos) y el comando `system_info`; verificar con un test que la normalización a "Windows 11" funciona y con un test `#[ignore]` que devuelve datos reales

## 2. Interfaz

- [x] 2.1 Crear `src/modules/health-check.ts` con datos del equipo, tarjetas independientes y botones "Revisar" con `navigate`; registrarlo el primero; verificar `npx tsc --noEmit`, `npx vite build` y una captura con datos simulados

## 3. Integración y documentación

- [x] 3.1 Abrir la app real y comprobar que cada tarjeta coincide con su módulo y que "Revisar" abre el módulo correcto
- [x] 3.2 Actualizar `README.md`; verificar que coincide con el código

## Workflow follow-up

- Probar en la app real y archivar con `/opsx:archive` después de `add-software-updater`.
