# Tasks

## 1. Backend

- [x] 1.1 Exponer la carpeta de datos en `log.rs` y crear `tweaks.rs` con el catálogo del diseño; verificar con tests que los ids son únicos y que `admin` se deriva de `HKLM`
- [x] 1.2 Implementar el almacén de originales (`tweaks.json`) con carga tolerante a errores; verificar con un test de ida y vuelta en una carpeta temporal
- [x] 1.3 Implementar estado, aplicar y restaurar; verificar con un test sobre `HKCU\Software\DonLimpiezasTest` que aplicar guarda el original, aplicar dos veces no lo sobrescribe y restaurar borra la clave creada
- [x] 1.4 Exponer `list_tweaks`, `apply_tweaks` y `revert_tweaks` con validación de ids y admin; verificar que `cargo build` compila sin warnings

## 2. Interfaz

- [x] 2.1 Crear `src/modules/performance-optimizer.ts` y registrarlo entre Bloatware Remover y Startup Manager; verificar `npx tsc --noEmit`, `npx vite build` y una captura con datos simulados

## 3. Integración y documentación

- [x] 3.1 Aplicar y restaurar `advertising_id` en la app real y comprobarlo en Configuración > Privacidad
- [x] 3.2 Actualizar `README.md`; verificar que coincide con el catálogo

## Workflow follow-up

- Probar en la app real y archivar con `/opsx:archive`.
