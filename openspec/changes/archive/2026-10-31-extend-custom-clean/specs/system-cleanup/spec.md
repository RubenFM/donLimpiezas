# Spec Delta

## MODIFIED Requirements

### Requirement: Categorías de limpieza

El sistema SHALL ofrecer las categorías de la tabla anterior, cada una con identificador, nombre, descripción, grupo e indicación de si requiere administrador. Las cachés de navegador SHALL cubrir todos los perfiles y MUST NOT incluir historial, contraseñas, cookies ni sesiones.

#### Scenario: Navegador con varios perfiles
- **WHEN** Chrome tiene los perfiles `Default` y `Profile 1`
- **THEN** la categoría `chrome_cache` incluye las carpetas de caché de ambos perfiles

#### Scenario: Navegador no instalado
- **WHEN** la carpeta de datos de un navegador no existe
- **THEN** su categoría se muestra con 0 bytes y 0 elementos

#### Scenario: Aplicación no instalada
- **WHEN** Spotify no está instalado
- **THEN** la categoría `spotify_cache` se muestra con 0 bytes y 0 elementos

## ADDED Requirements

### Requirement: Categorías de privacidad

El sistema SHALL ofrecer en el grupo "Privacidad" el borrado de historial y cookies de los navegadores basados en Chromium (todos los perfiles), de las cookies de Firefox y de los documentos recientes. Estas categorías MUST aparecer desmarcadas por defecto y SHALL mostrar un aviso de sus consecuencias. MUST NOT borrar contraseñas, marcadores ni el historial de Firefox.

#### Scenario: Análisis con privacidad
- **WHEN** el usuario analiza y hay cookies de Chrome
- **THEN** "Historial y cookies de Chrome" aparece con su tamaño, desmarcada y con el aviso "Cerrará tu sesión en las webs"

#### Scenario: Marcadores intactos
- **WHEN** se limpia "Historial y cookies de Chrome"
- **THEN** los marcadores y las contraseñas guardadas de Chrome siguen disponibles

### Requirement: Aviso de programa abierto

Cuando un programa asociado a una categoría se está ejecutando durante el análisis, el sistema SHALL mostrar en esa categoría la etiqueta "Abierto" y un aviso indicando que conviene cerrarlo para limpiarla por completo. La categoría MUST seguir pudiéndose seleccionar.

#### Scenario: Chrome abierto
- **WHEN** el usuario analiza con Chrome abierto
- **THEN** las categorías de Chrome muestran "Abierto" y el aviso
- **AND** se pueden seleccionar y limpiar, omitiendo los archivos en uso
