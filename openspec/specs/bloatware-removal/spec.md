# bloatware-removal Specification

## Purpose

Detectar y desinstalar aplicaciones preinstaladas de Windows (paquetes AppX) que el usuario no necesita, limitándose a un catálogo revisado para no romper el sistema.

Implementado en `src-tauri/src/debloat.rs` (comandos `list_bloat`, `create_restore_point` y `remove_bloat`) y en la pestaña "Apps preinstaladas" de `src/main.ts`.

## Requirements

### Requirement: Catálogo cerrado de apps

El sistema SHALL definir un catálogo de paquetes eliminables, cada uno con nombre de paquete, nombre visible, grupo ("Apps de Microsoft", "Xbox y juegos", "Apps promocionadas de terceros") y una nota opcional de advertencia.

El catálogo MUST NOT incluir Microsoft Store, runtimes (VCLibs, .NET, UI.Xaml), componentes del shell, Seguridad de Windows ni paquetes de los que dependan otros (p. ej. `Microsoft.XboxIdentityProvider`, `Microsoft.Xbox.TCUI`).

#### Scenario: App con advertencia
- **WHEN** se lista `Microsoft.XboxGamingOverlay`
- **THEN** se muestra la nota "Usada para grabar pantalla con Win+G."

### Requirement: Detección de apps presentes

El sistema SHALL determinar para cada app del catálogo si está instalada para el usuario actual (`Get-AppxPackage`) y, cuando se ejecuta como administrador, si está aprovisionada en la imagen de Windows (`Get-AppxProvisionedPackage -Online`). Solo SHALL mostrar las apps instaladas o aprovisionadas.

#### Scenario: App solo aprovisionada
- **WHEN** como administrador, una app está aprovisionada pero no instalada para el usuario
- **THEN** se muestra con la etiqueta "Solo en la imagen"

#### Scenario: Ninguna app presente
- **WHEN** ninguna app del catálogo está instalada ni aprovisionada
- **THEN** se muestra un mensaje indicando que no se ha encontrado bloatware

### Requirement: Solo paquetes del catálogo

El sistema MUST rechazar cualquier solicitud de eliminación que contenga un identificador que no pertenezca al catálogo, sin eliminar nada. Los nombres de paquete SHALL incrustarse en los scripts de PowerShell solo después de esta validación y escapados como literales.

#### Scenario: Identificador no permitido
- **WHEN** se invoca `remove_bloat` con `Microsoft.WindowsStore`
- **THEN** el comando devuelve el error "Paquete no permitido" y no se ejecuta ningún script

### Requirement: Ámbito de eliminación

El sistema SHALL quitar las apps seleccionadas para el usuario actual por defecto. Cuando el usuario marca "Para todos los usuarios", SHALL quitarlas para todos los usuarios y eliminar su aprovisionamiento de la imagen, y MUST exigir permisos de administrador para ello.

#### Scenario: Todos los usuarios sin permisos
- **WHEN** se solicita quitar para todos los usuarios sin ejecutar como administrador
- **THEN** el comando devuelve un error y no se elimina nada

### Requirement: Punto de restauración

Cuando el usuario marca "Crear punto de restauración" (opción marcada por defecto y disponible solo como administrador), el sistema SHALL crear un punto de restauración antes de eliminar apps. Si falla, SHALL preguntar al usuario si desea continuar igualmente.

#### Scenario: Límite de un punto cada 24 horas
- **WHEN** Windows omite el punto de restauración porque ya se creó uno en las últimas 24 horas
- **THEN** se muestra el aviso de Windows y la eliminación continúa

#### Scenario: Protección del sistema desactivada
- **WHEN** la creación del punto de restauración falla
- **THEN** se pregunta al usuario si quiere continuar y, si cancela, no se elimina nada

### Requirement: Confirmación y resultado por app

El sistema SHALL pedir confirmación listando las apps y el ámbito, y SHALL informar del resultado de cada app de forma independiente: un fallo en una no detiene las demás.

#### Scenario: Fallo parcial
- **WHEN** se quitan tres apps y una falla
- **THEN** las otras dos se eliminan y se muestra "2 de 3 apps quitadas" con el error de la que falló

### Requirement: Registro de cambios

El sistema SHALL añadir una línea por app procesada a `%LOCALAPPDATA%\DonLimpiezas\debloat.log` con fecha ISO 8601, nombre del paquete y `OK` o el mensaje de error.

#### Scenario: Registro tras eliminar
- **WHEN** se quita `Microsoft.BingNews` correctamente
- **THEN** el log contiene una línea con la fecha, `Microsoft.BingNews` y `OK`
