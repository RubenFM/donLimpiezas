# Tasks

## 1. Backend

- [x] 1.1 Exponer el espacio libre de un disco en `sysinfo.rs`, añadir `Target::ComponentStore`, `size_known` y la categoría; verificar con tests que es la única de tamaño desconocido, requiere admin y va desmarcada

## 2. Interfaz

- [x] 2.1 Mostrar "—" y "tamaño desconocido" en Custom Clean y "aprox." en el resultado; verificar `npx tsc --noEmit`, `npx vite build` y una captura

## 3. Integración y documentación

- [x] 3.1 Ejecutar la limpieza como administrador en la app real y comprobar el espacio liberado
- [x] 3.2 Actualizar la tabla de `specs/system-cleanup` y `README.md`; verificar que coinciden con `categories()`

## Workflow follow-up

- Probar en la app real y archivar con `/opsx:archive`.
