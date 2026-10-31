# Tasks

## 1. Backend

- [x] 1.1 Añadir los orígenes `ScheduledTask` y `Service` y el filtro de elementos de sistema; verificar con tests del filtro (System32, `%windir%`, Defender, lista fija)
- [x] 1.2 Listar tareas y servicios con una llamada a PowerShell e integrarlos en `list()`; verificar con el test `#[ignore]` de entradas reales que aparecen EasyTune y Google Update y no Defender
- [x] 1.3 Implementar activar/desactivar tareas y servicios con validación, `services.json` y log; verificar con tests que rechaza `WinDefend` y entradas desconocidas

## 2. Interfaz

- [x] 2.1 Notas por origen en `startup-manager.ts`; verificar `npx tsc --noEmit`, `npx vite build` y una captura

## 3. Integración y documentación

- [ ] 3.1 Desactivar y reactivar en la app real una tarea y un servicio y comprobarlo en el Programador de tareas y en Servicios
- [x] 3.2 Actualizar `README.md`; verificar que coincide con el código

## Workflow follow-up

- Probar en la app real y archivar con `/opsx:archive`.
