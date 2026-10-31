# Tasks

## 1. Base compartida

- [x] 1.1 Mover `expand_env` y la extracción del ejecutable a `src-tauri/src/cmdline.rs` como `split(command) -> Option<(exe, args)>`, actualizar `startup.rs` y mover sus tests; verificar que `cargo test` pasa
- [x] 1.2 Añadir la reescritura de MSI (`/I{GUID}` o `/X{GUID}` → `msiexec.exe /X{GUID}`) con tests unitarios

## 2. Backend

- [x] 2.1 Implementar `uninstaller::list()` leyendo las tres vistas con el filtrado del diseño; verificar con un test `#[ignore]` que lista programas reales sin actualizaciones ni componentes del sistema
- [x] 2.2 Implementar `uninstaller::uninstall(id)` con validación, `ShellExecuteExW` + espera, comprobación posterior y `uninstall.log`; verificar con un test que rechaza ids inexistentes
- [x] 2.3 Exponer `list_programs` y `uninstall_program` en `lib.rs` y verificar que `cargo build` compila sin warnings

## 3. Interfaz

- [x] 3.1 Crear `src/modules/uninstaller.ts` con búsqueda, orden, selección, cola con progreso y resultados, y registrarlo entre Custom Clean y Bloatware Remover; verificar `npx tsc --noEmit`, `npx vite build` y una captura con datos simulados

## 4. Integración y documentación

- [ ] 4.1 Desinstalar un programa real de prueba desde la app y comprobar que desaparece de la lista y de Configuración
- [x] 4.2 Actualizar `README.md` (funciones y estructura); verificar que coincide con el código

## Workflow follow-up

- Probar en la app real y archivar con `/opsx:archive`.
