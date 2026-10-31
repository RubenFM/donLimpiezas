# Spec Delta

## Purpose

Mostrar qué carpetas y archivos ocupan el espacio de un disco para que el usuario sepa dónde actuar.

## ADDED Requirements

### Requirement: Análisis de un disco o carpeta

El sistema SHALL analizar el disco fijo o la carpeta que el usuario elija, sumando el tamaño de cada carpeta, en segundo plano con progreso (archivos y carpetas recorridos) y cancelación. MUST NOT seguir enlaces ni uniones, MUST contar como 0 bytes los archivos que solo están en la nube y MUST continuar si una carpeta no se puede leer, indicando cuántas no se pudieron leer.

#### Scenario: Carpetas sin permiso
- **WHEN** la app no es administradora y analiza C:
- **THEN** el análisis termina e indica cuántas carpetas no se pudieron leer

#### Scenario: OneDrive con archivos en la nube
- **WHEN** una carpeta de OneDrive tiene archivos solo en la nube
- **THEN** esos archivos no suman espacio en disco

### Requirement: Navegación por el resultado

El sistema SHALL mostrar el contenido de la carpeta actual (subcarpetas y los archivos más grandes, más una entrada con el resto de archivos) ordenado por tamaño, con el porcentaje respecto a la carpeta y una barra proporcional. SHALL permitir entrar en una subcarpeta y volver a cualquier carpeta anterior mediante la ruta de migas, sin volver a analizar.

#### Scenario: Entrar en una carpeta
- **WHEN** el usuario pulsa en `Users` tras analizar C:
- **THEN** se muestran las carpetas de `C:\Users` ordenadas por tamaño de inmediato

### Requirement: Abrir en el Explorador

El sistema SHALL permitir abrir la carpeta actual en el Explorador de archivos. MUST rechazar rutas que no pertenezcan al último análisis. El módulo MUST NOT borrar ni modificar archivos.

#### Scenario: Abrir carpeta
- **WHEN** el usuario pulsa "Abrir en el Explorador" en `C:\Users\ruben\Videos`
- **THEN** se abre el Explorador en esa carpeta
