# startup-programs Specification

## Purpose
Mostrar qué programas se ejecutan al iniciar sesión en Windows y permitir activarlos o desactivarlos de forma reversible y coherente con el Administrador de tareas.

## Requirements

### Requirement: Listado de programas de inicio

El sistema SHALL listar las entradas de inicio de las claves `Run` del usuario y del equipo (vistas de 64 y 32 bits) y de las carpetas de Inicio del usuario y común, mostrando para cada una su nombre, comando o destino, origen y estado.

#### Scenario: Entrada en el registro del usuario
- **WHEN** existe el valor `Spotify` en `HKCU\Software\Microsoft\Windows\CurrentVersion\Run`
- **THEN** aparece una entrada "Spotify" con su comando y el origen "Registro (usuario)"

#### Scenario: Acceso directo en la carpeta de Inicio
- **WHEN** existe `Discord.lnk` en la carpeta de Inicio del usuario
- **THEN** aparece una entrada "Discord" con el destino del acceso directo y el origen "Carpeta de Inicio (usuario)"

#### Scenario: Sin entradas
- **WHEN** no hay ninguna entrada de inicio
- **THEN** se muestra un mensaje indicando que no hay programas de inicio

### Requirement: Estado coherente con el Administrador de tareas

El sistema SHALL determinar si una entrada está activada o desactivada leyendo los mismos datos que usa el Administrador de tareas de Windows (`StartupApproved`). Una entrada sin dato de aprobación SHALL considerarse activada.

#### Scenario: Desactivada desde el Administrador de tareas
- **WHEN** el usuario ha desactivado "Teams" en el Administrador de tareas
- **THEN** Don Limpiezas muestra "Teams" como desactivada

### Requirement: Activar y desactivar de forma reversible

El sistema SHALL permitir activar y desactivar cada entrada sin borrar ni modificar la entrada original, de modo que el cambio se pueda deshacer desde Don Limpiezas o desde el Administrador de tareas.

#### Scenario: Desactivar una entrada
- **WHEN** el usuario desactiva "Spotify"
- **THEN** Spotify no se ejecuta en el siguiente inicio de sesión
- **AND** el valor `Spotify` sigue existiendo en la clave `Run`
- **AND** el Administrador de tareas lo muestra como deshabilitado

#### Scenario: Reactivar una entrada
- **WHEN** el usuario activa de nuevo "Spotify"
- **THEN** Spotify vuelve a ejecutarse en el siguiente inicio de sesión

### Requirement: Entradas del equipo requieren administrador

El sistema SHALL marcar las entradas de origen equipo (`HKLM` y carpeta de Inicio común) con la etiqueta "Admin" y MUST impedir cambiarlas cuando la app no se ejecuta como administrador.

#### Scenario: Sin permisos
- **WHEN** la app se ejecuta sin administrador
- **THEN** las entradas del equipo se muestran con su estado pero su interruptor está deshabilitado

#### Scenario: Solicitud sin permisos
- **WHEN** se solicita cambiar una entrada del equipo sin ser administrador
- **THEN** el sistema devuelve un error y no modifica nada

### Requirement: Solo entradas detectadas

El sistema MUST rechazar cambios sobre entradas que no estén en el listado actual (mismo origen y nombre), sin modificar nada.

#### Scenario: Entrada inexistente
- **WHEN** se solicita desactivar una entrada que ya no existe en su origen
- **THEN** el sistema devuelve un error y no escribe en el registro

### Requirement: Registro de cambios de inicio

El sistema SHALL añadir una línea a `%LOCALAPPDATA%\DonLimpiezas\startup.log` por cada cambio con fecha ISO 8601, origen, nombre y nuevo estado.

#### Scenario: Registro tras desactivar
- **WHEN** se desactiva "Spotify"
- **THEN** el log contiene una línea con la fecha, "Registro (usuario)", "Spotify" y "desactivado"

### Requirement: Tareas programadas de inicio

El sistema SHALL listar las tareas programadas que no son de Windows y se ejecutan al iniciar sesión o al arrancar el equipo, con su nombre, ejecutable, autor y estado, y SHALL permitir activarlas o desactivarlas con administrador.

#### Scenario: Tarea de un fabricante
- **WHEN** existe la tarea `\EasyTune` de GIGABYTE con disparador de inicio de sesión
- **THEN** aparece en el origen "Tarea programada" y se puede desactivar

#### Scenario: Tarea de Windows
- **WHEN** una tarea fuera de `\Microsoft\` ejecuta un programa de `C:\Windows\System32`
- **THEN** no aparece en la lista

### Requirement: Servicios de terceros

El sistema SHALL listar los servicios con inicio automático cuyo ejecutable está fuera de la carpeta de Windows, más los que la propia app pasó a inicio manual, y SHALL permitir alternar entre inicio automático y manual con administrador, sin deshabilitarlos nunca. El cambio SHALL conservar el inicio retrasado si lo tenía. Los servicios de Microsoft Defender MUST NOT aparecer ni poder cambiarse.

#### Scenario: Desactivar un servicio de actualización
- **WHEN** el usuario desactiva "Google updater service"
- **THEN** el servicio pasa a inicio manual, sigue apareciendo en la lista como desactivado y se puede volver a activar

#### Scenario: Antivirus
- **WHEN** Microsoft Defender está instalado fuera de la carpeta de Windows
- **THEN** sus servicios no aparecen

#### Scenario: Validación
- **WHEN** se solicita cambiar un servicio que no está en la lista actual, como `WinDefend`
- **THEN** el sistema devuelve un error sin modificar nada
