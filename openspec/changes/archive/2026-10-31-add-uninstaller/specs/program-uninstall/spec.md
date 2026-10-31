# Spec Delta

## Purpose

Ver los programas instalados en Windows con su tamaño y fecha, y desinstalarlos de forma ordenada usando el desinstalador oficial de cada uno.

## ADDED Requirements

### Requirement: Inventario de programas

El sistema SHALL listar los programas registrados en las claves `Uninstall` del equipo (vistas de 64 y 32 bits) y del usuario, mostrando nombre, editor, versión, tamaño estimado y fecha de instalación cuando existan. MUST ocultar las entradas sin nombre o sin desinstalador, los componentes del sistema y las actualizaciones.

#### Scenario: Programa con todos los datos
- **WHEN** 7-Zip está instalado con tamaño y fecha registrados
- **THEN** aparece "7-Zip" con su editor, versión, tamaño y fecha

#### Scenario: Actualización de un programa
- **WHEN** una entrada tiene `ParentKeyName` o `ReleaseType` de actualización
- **THEN** no aparece en la lista

### Requirement: Búsqueda y orden

El sistema SHALL permitir filtrar la lista por texto contenido en el nombre o el editor, sin distinguir mayúsculas, y ordenarla por nombre, tamaño (los que no lo tienen al final) o fecha de instalación (más recientes primero).

#### Scenario: Buscar por editor
- **WHEN** el usuario escribe "riot"
- **THEN** solo se muestran los programas cuyo nombre o editor contiene "riot"

### Requirement: Desinstalación con el desinstalador oficial

El sistema SHALL desinstalar cada programa ejecutando su desinstalador registrado con su interfaz visible, permitiendo que Windows pida elevación si el desinstalador la necesita. Para instaladores MSI SHALL usar `msiexec /X` con el código del producto. MUST NOT ejecutar desinstalaciones silenciosas.

#### Scenario: Programa MSI registrado con /I
- **WHEN** el desinstalador registrado es `MsiExec.exe /I{GUID}`
- **THEN** se ejecuta `msiexec.exe /X{GUID}`

#### Scenario: Desinstalador que requiere administrador
- **WHEN** la app no es administrador y el desinstalador lo requiere
- **THEN** Windows muestra el diálogo de UAC y, si se acepta, el desinstalador se abre

### Requirement: Desinstalación en cola con confirmación

El sistema SHALL pedir confirmación listando los programas seleccionados y desinstalarlos de uno en uno, esperando a que termine cada desinstalador antes de lanzar el siguiente. Tras cada uno SHALL comprobar si el programa sigue registrado e informar del resultado.

#### Scenario: Desinstalación completada
- **WHEN** el desinstalador termina y la entrada desaparece del registro
- **THEN** se informa "Desinstalado"

#### Scenario: Usuario cancela el asistente
- **WHEN** el desinstalador termina pero el programa sigue registrado
- **THEN** se informa que sigue instalado y se continúa con el siguiente

### Requirement: Solo programas detectados

El sistema MUST rechazar la desinstalación de un identificador que no esté en el inventario actual, sin ejecutar nada.

#### Scenario: Programa ya desinstalado
- **WHEN** se solicita desinstalar un programa que ya no está registrado
- **THEN** el sistema devuelve un error y no ejecuta ningún desinstalador

### Requirement: Registro de desinstalaciones

El sistema SHALL añadir una línea a `%LOCALAPPDATA%\DonLimpiezas\uninstall.log` por cada programa procesado con fecha ISO 8601, nombre, versión y resultado.

#### Scenario: Registro tras desinstalar
- **WHEN** se desinstala "7-Zip 24.08"
- **THEN** el log contiene una línea con la fecha, "7-Zip", "24.08" y "desinstalado"
