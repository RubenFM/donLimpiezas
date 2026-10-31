# software-updates Specification

## Purpose
Mantener actualizados los programas instalados usando winget y los instaladores oficiales de cada programa.

## Requirements

### Requirement: Detección de actualizaciones

El sistema SHALL listar los programas para los que winget ofrece una versión más reciente, incluidos los de versión instalada desconocida, mostrando nombre, identificador, versión instalada y versión disponible. La lectura MUST funcionar con la salida de winget en cualquier idioma.

#### Scenario: Actualizaciones disponibles
- **WHEN** Git 2.45.1 está instalado y winget ofrece 2.56.0.4
- **THEN** aparece "Git" con `Git.Git`, "2.45.1" y "2.56.0.4"

#### Scenario: winget no disponible
- **WHEN** winget no está instalado
- **THEN** se muestra un error que explica cómo instalarlo desde Microsoft Store

### Requirement: Actualización en cola con confirmación

El sistema SHALL actualizar los programas seleccionados tras confirmación, de uno en uno, con winget y el instalador oficial, sin forzar el modo silencioso, informando del progreso y del resultado de cada programa. Un fallo MUST NOT detener los siguientes.

#### Scenario: Una actualización falla
- **WHEN** se actualizan tres programas y el segundo falla
- **THEN** se actualizan el primero y el tercero y se muestra el error del segundo

### Requirement: Solo programas detectados

El sistema MUST rechazar la actualización de un identificador que no esté en la última lista de actualizaciones, sin ejecutar winget.

#### Scenario: Identificador desconocido
- **WHEN** se solicita actualizar `Malware.Pkg`
- **THEN** el sistema devuelve un error sin ejecutar nada

### Requirement: Registro de actualizaciones

El sistema SHALL añadir una línea a `%LOCALAPPDATA%\DonLimpiezas\updates.log` por cada programa procesado con fecha ISO 8601, identificador, versiones y resultado.

#### Scenario: Registro
- **WHEN** se actualiza `Git.Git` correctamente
- **THEN** el log contiene una línea con la fecha, `Git.Git`, "2.45.1 → 2.56.0.4" y "actualizado"
