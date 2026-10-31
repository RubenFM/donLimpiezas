# Spec Delta

## ADDED Requirements

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
