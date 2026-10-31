# cloud-storage-cleanup Specification

## Purpose
Recuperar espacio en disco dejando "solo en la nube" archivos de OneDrive y Dropbox que ya están guardados en la nube, sin borrarlos.

## Requirements

### Requirement: Detección de carpetas sincronizadas

El sistema SHALL detectar las carpetas sincronizadas de todas las cuentas de OneDrive y de Dropbox del usuario y mostrar, si no hay ninguna, que no se ha encontrado ningún servicio compatible.

#### Scenario: OneDrive personal
- **WHEN** el usuario tiene OneDrive personal en `C:\Users\ruben\OneDrive`
- **THEN** esa carpeta aparece como "OneDrive (personal)"

### Requirement: Archivos descargados en local

El sistema SHALL listar los archivos de las carpetas sincronizadas que están descargados en el equipo, mayores o iguales al tamaño mínimo elegido y, si se indica, sin modificar desde hace un número de días, ordenados por tamaño. SHALL indicar los marcados como "Mantener siempre en este dispositivo". MUST NOT leer el contenido de ningún archivo ni provocar descargas.

#### Scenario: Archivo solo en la nube
- **WHEN** un archivo ya está solo en la nube
- **THEN** no aparece en la lista

#### Scenario: Archivo fijado
- **WHEN** un archivo está marcado como "Mantener siempre en este dispositivo"
- **THEN** aparece con la etiqueta "Fijado" y desmarcado

### Requirement: Liberar espacio sin borrar

El sistema SHALL marcar como "solo en la nube" los archivos seleccionados tras confirmación, sin borrarlos, y SHALL informar de cuántos se liberaron. MUST rechazar sin hacer nada cualquier ruta que no esté en el último análisis o fuera de una carpeta sincronizada.

#### Scenario: Liberar un vídeo
- **WHEN** se libera un vídeo de 2 GB
- **THEN** el vídeo sigue apareciendo en OneDrive, ocupa 0 bytes en el disco y se descarga al abrirlo

#### Scenario: Ruta ajena
- **WHEN** se solicita liberar `C:\Windows\explorer.exe`
- **THEN** el sistema devuelve un error sin modificar nada

### Requirement: Registro de espacio liberado

El sistema SHALL añadir una línea a `%LOCALAPPDATA%\DonLimpiezas\cloud.log` por cada archivo liberado con fecha ISO 8601 y ruta.

#### Scenario: Registro
- **WHEN** se libera `C:\Users\ruben\OneDrive\Vídeos\viaje.mp4`
- **THEN** el log contiene una línea con la fecha y esa ruta
