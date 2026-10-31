# Tasks

## 1. Dependencias y utilidades

- [ ] 1.1 Añadir `winreg` y `chrono` (solo feature `clock`) a `src-tauri/Cargo.toml` y verificar que `cargo build` sigue compilando sin warnings
- [ ] 1.2 Crear `src-tauri/src/log.rs` con `append(file, line)` que escribe en `%LOCALAPPDATA%\DonLimpiezas\<file>` con fecha ISO 8601, y verificar con un test unitario que crea la carpeta y añade líneas sin sobrescribir

## 2. Lógica de `StartupApproved` y parseo

- [ ] 2.1 Implementar en `startup.rs` la interpretación de los bytes de aprobación (par = activado, impar = desactivado, ausente = activado) y la generación de los 12 bytes para activar/desactivar; verificar con tests unitarios de `cargo test`
- [ ] 2.2 Implementar la extracción del ejecutable desde un comando de `Run` (comillas, sin comillas hasta `.exe`, variables de entorno) y verificar con tests unitarios que cubran `"C:\Program Files\X\x.exe" --min`, `C:\Tools\y.exe /s` y `%ProgramFiles%\Z\z.exe`

## 3. Backend: listado y cambio de estado

- [ ] 3.1 Implementar la lectura de los cinco orígenes del diseño con su estado y verificar con `cargo test` (marcado `#[ignore]` si toca el registro real) que se listan las entradas de `HKCU\…\Run` de la máquina
- [ ] 3.2 Resolver destino de `.lnk` y editor con una única llamada a `ps::run`, tolerando fallos; verificar que un `.lnk` de la carpeta de Inicio muestra su destino
- [ ] 3.3 Implementar `set_enabled(origen, nombre, activado)` con validación de existencia, comprobación de admin para orígenes de equipo y escritura en `startup.log`; verificar con tests que rechaza entradas inexistentes y orígenes de equipo sin admin
- [ ] 3.4 Exponer los comandos `list_startup` y `set_startup_enabled` en `lib.rs` y verificar que `cargo build` compila sin warnings

## 4. Interfaz

- [ ] 4.1 Crear `src/modules/startup-manager.ts` con su icono, botón "Actualizar" y la lista, registrarlo en `src/modules/index.ts` entre Performance Optimizer y Software Updater, y verificar que `npx tsc --noEmit` y `npx vite build` pasan
- [ ] 4.2 Renderizar las entradas agrupadas por origen con interruptor, etiqueta "Admin", destino y editor; el interruptor revierte su estado si el backend devuelve error. Verificar con `npm run tauri dev` que la lista coincide con la pestaña "Inicio" del Administrador de tareas
- [ ] 4.3 Añadir los estilos del interruptor en `src/styles.css` con soporte de modo claro y oscuro, y verificarlo visualmente en ambos modos

## 5. Integración y documentación

- [ ] 5.1 Desactivar una entrada de usuario desde Don Limpiezas y comprobar que el Administrador de tareas la muestra deshabilitada; reactivarla desde el Administrador de tareas y comprobar que Don Limpiezas lo refleja tras "Actualizar"
- [ ] 5.2 Actualizar la sección "Funciones" y "Estructura" del `README.md` con el gestor de inicio y verificar que describe los orígenes cubiertos

## Workflow follow-up

- Archivar el cambio con `/opsx:archive` tras revisar la implementación, para fusionar `startup-programs` en `openspec/specs/`.
