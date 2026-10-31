# Spec Delta

## Purpose

Encontrar archivos con contenido idéntico en las carpetas del usuario y recuperar espacio enviando las copias sobrantes a la papelera, sin riesgo de perder el original.

## ADDED Requirements

### Requirement: Carpetas a analizar

El sistema SHALL proponer por defecto las carpetas Documentos, Descargas, Imágenes, Vídeos y Música del usuario en su ubicación real, y SHALL permitir añadir carpetas con el selector del sistema y quitar cualquiera de la lista.

#### Scenario: Documentos en OneDrive
- **WHEN** la carpeta Documentos del usuario está redirigida a OneDrive
- **THEN** la lista por defecto muestra la ruta de OneDrive

### Requirement: Detección por contenido

El sistema SHALL considerar duplicados los archivos de igual tamaño, mayores o iguales al mínimo elegido, cuyo contenido completo tenga el mismo hash, independientemente de su nombre. MUST NOT seguir enlaces simbólicos ni uniones y MUST NOT leer archivos que solo estén en la nube, para no provocar su descarga.

#### Scenario: Mismo contenido, distinto nombre
- **WHEN** `foto.jpg` y `foto (1).jpg` tienen el mismo contenido
- **THEN** aparecen en el mismo grupo

#### Scenario: Mismo tamaño, distinto contenido
- **WHEN** dos archivos tienen el mismo tamaño pero distinto contenido
- **THEN** no aparecen como duplicados

#### Scenario: Archivo solo en la nube
- **WHEN** un archivo de OneDrive no está descargado
- **THEN** se ignora y no se descarga

### Requirement: Progreso y cancelación

El sistema SHALL analizar en segundo plano informando del progreso (archivos encontrados y archivos comparados) y SHALL permitir cancelar el análisis en cualquier momento.

#### Scenario: Cancelar
- **WHEN** el usuario pulsa "Cancelar" durante el análisis
- **THEN** el análisis se detiene y no se muestran resultados parciales

### Requirement: Resultados y selección

El sistema SHALL mostrar los grupos ordenados por espacio recuperable, con ruta y fecha de modificación de cada archivo, y SHALL ofrecer seleccionar automáticamente todas las copias excepto la más antigua de cada grupo.

#### Scenario: Selección automática
- **WHEN** el usuario pulsa "Conservar el más antiguo"
- **THEN** en cada grupo quedan seleccionados todos los archivos menos el de fecha de modificación más antigua

### Requirement: Eliminación segura a la papelera

El sistema SHALL enviar los archivos seleccionados a la papelera de reciclaje tras confirmación. MUST rechazar la operación sin eliminar nada si incluye archivos que no estén en el último análisis o si dejaría algún grupo sin ninguna copia.

#### Scenario: Seleccionar todas las copias
- **WHEN** el usuario selecciona todos los archivos de un grupo
- **THEN** la app impide eliminarlos e indica que debe conservar al menos uno

#### Scenario: Recuperar desde la papelera
- **WHEN** se eliminan duplicados
- **THEN** los archivos aparecen en la papelera y se pueden restaurar

### Requirement: Registro de duplicados eliminados

El sistema SHALL añadir una línea a `%LOCALAPPDATA%\DonLimpiezas\duplicates.log` por cada archivo enviado a la papelera, con fecha ISO 8601 y ruta.

#### Scenario: Registro
- **WHEN** se envía `C:\Users\ruben\Downloads\foto (1).jpg` a la papelera
- **THEN** el log contiene una línea con la fecha y esa ruta
