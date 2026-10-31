# Proposal

## Why

Desinstalar desde Configuración de Windows es lento, no deja ordenar por tamaño ni fecha y obliga a desinstalar los programas de uno en uno, volviendo a buscar cada vez.

## What Changes

- Nuevo módulo **Uninstaller** en la barra lateral, entre Custom Clean y Bloatware Remover.
- Inventario de programas instalados (claves `Uninstall` de HKLM 64/32 bits y HKCU) con nombre, editor, versión, tamaño estimado y fecha de instalación. Se ocultan componentes del sistema y actualizaciones.
- Búsqueda por nombre o editor y orden por nombre, tamaño o fecha.
- Desinstalar varios programas seguidos: se lanza el **desinstalador oficial** de cada uno, de uno en uno, siempre con su asistente (nunca en modo silencioso), y se comprueba si el programa desapareció.
- Registro en `%LOCALAPPDATA%\DonLimpiezas\uninstall.log`.
- Fuera de alcance: apps de Microsoft Store (las del catálogo están en Bloatware Remover), desinstalación silenciosa y búsqueda de restos tras desinstalar.

## Capabilities

### New Capabilities
- `program-uninstall`: inventario de programas instalados y desinstalación mediante su desinstalador oficial.

### Modified Capabilities

## Impact

- Nuevo `src-tauri/src/uninstaller.rs` y comandos `list_programs` y `uninstall_program`.
- La extracción del ejecutable de un comando (`startup.rs`) pasa a un módulo compartido `cmdline.rs`.
- `windows-sys`: features `Win32_System_Registry` y `Win32_System_Threading` para `ShellExecuteExW` y esperar al desinstalador.
- Nuevo `src/modules/uninstaller.ts`.
