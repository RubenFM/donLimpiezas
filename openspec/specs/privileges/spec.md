# privileges Specification

## Purpose

Permitir que la app funcione sin permisos de administrador y elevarse bajo demanda cuando el usuario necesita las funciones que los requieren.

Implementado en `src-tauri/src/system.rs` (comandos `is_admin` y `relaunch_as_admin`).

## Requirements

### Requirement: Ejecución sin elevación

La app SHALL arrancar con los permisos del usuario que la lanza (`asInvoker`) y SHALL detectar si se está ejecutando como administrador.

#### Scenario: Usuario estándar
- **WHEN** la app se abre sin elevación
- **THEN** se muestra un aviso indicando que algunas categorías y opciones están desactivadas

### Requirement: Reinicio como administrador

El sistema SHALL ofrecer un botón "Reiniciar como administrador" que lance una nueva instancia elevada mediante UAC y cierre la actual. Si el usuario cancela el diálogo de UAC, la instancia actual MUST seguir abierta y SHALL mostrar un error.

#### Scenario: UAC aceptado
- **WHEN** el usuario pulsa el botón y acepta el diálogo de UAC
- **THEN** se abre una instancia elevada y la actual se cierra

#### Scenario: UAC cancelado
- **WHEN** el usuario cancela el diálogo de UAC
- **THEN** la app sigue abierta y muestra "No se concedieron permisos de administrador."

### Requirement: Ejecución de PowerShell sin ventana

Las operaciones de sistema que usan PowerShell SHALL ejecutarse sin abrir ventanas de consola, con salida en UTF-8 y pasando el script codificado (`-EncodedCommand`) para evitar problemas de escapado.

#### Scenario: Nombres con caracteres no ASCII
- **WHEN** PowerShell devuelve texto con tildes
- **THEN** la app lo muestra correctamente
