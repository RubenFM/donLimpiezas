# Spec Delta

## Purpose

Desactivar telemetría, anuncios y funciones innecesarias de Windows mediante ajustes revisados que siempre se pueden deshacer.

## ADDED Requirements

### Requirement: Catálogo cerrado de ajustes

El sistema SHALL ofrecer únicamente los ajustes de su catálogo, cada uno con nombre, descripción, grupo, indicación de si requiere administrador y aviso opcional. El sistema MUST rechazar cualquier identificador que no pertenezca al catálogo sin modificar nada.

#### Scenario: Identificador desconocido
- **WHEN** se solicita aplicar `borrar-system32`
- **THEN** el sistema devuelve un error y no modifica el registro

### Requirement: Estado de cada ajuste

El sistema SHALL mostrar para cada ajuste si está aplicado, es decir, si todos sus valores coinciden con los del ajuste, y si hay un valor original guardado que se pueda restaurar.

#### Scenario: Ajuste ya aplicado a mano
- **WHEN** el ID de publicidad ya estaba desactivado antes de usar la app
- **THEN** el ajuste aparece como aplicado y sin opción de restaurar

### Requirement: Aplicar con confirmación guardando el original

El sistema SHALL aplicar los ajustes seleccionados tras confirmación y, antes de escribir, MUST guardar el valor original de cada clave (o su ausencia) si no estaba ya guardado. Los ajustes que requieren administrador MUST rechazarse cuando la app no es administradora.

#### Scenario: Aplicar sin permisos
- **WHEN** se aplica "Telemetría de Windows" sin ser administrador
- **THEN** el sistema devuelve un error y no escribe nada

#### Scenario: Aplicar dos veces
- **WHEN** se aplica un ajuste, se restaura a mano en Windows y se vuelve a aplicar
- **THEN** el valor original guardado sigue siendo el de la primera vez

### Requirement: Restaurar valores originales

El sistema SHALL restaurar un ajuste devolviendo cada clave a su valor original guardado, o eliminándola si no existía, y olvidar después ese original. SHALL ofrecer restaurar un ajuste concreto o todos los guardados.

#### Scenario: Restaurar una clave que no existía
- **WHEN** se restaura un ajuste cuya clave no existía antes de aplicarlo
- **THEN** la clave se elimina y el ajuste deja de aparecer como aplicado

### Requirement: Registro de ajustes

El sistema SHALL añadir una línea a `%LOCALAPPDATA%\DonLimpiezas\tweaks.log` por cada ajuste aplicado o restaurado, con fecha ISO 8601, identificador y acción.

#### Scenario: Registro tras aplicar
- **WHEN** se aplica `advertising_id`
- **THEN** el log contiene una línea con la fecha, `advertising_id` y "aplicado"
