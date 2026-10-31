# driver-updates Specification

## Purpose
Mantener los controladores al día usando únicamente Windows Update y las herramientas oficiales de los fabricantes de tarjetas gráficas.

## Requirements

### Requirement: Inventario de controladores

El sistema SHALL listar los controladores instalados con nombre del dispositivo, clase, proveedor, versión y fecha, y SHALL permitir filtrarlos por texto.

#### Scenario: Tarjeta gráfica
- **WHEN** el equipo tiene una NVIDIA GeForce RTX 5070 Ti
- **THEN** aparece con proveedor "NVIDIA", su versión y su fecha

### Requirement: Actualizaciones solo de Windows Update

El sistema SHALL buscar actualizaciones de controladores únicamente en Windows Update y mostrar título, clase, fabricante, fecha y tamaño de descarga. MUST NOT descargar ni instalar controladores de ninguna otra fuente.

#### Scenario: Sin actualizaciones
- **WHEN** Windows Update no ofrece controladores
- **THEN** se indica que los controladores están al día

### Requirement: Instalación con administrador y punto de restauración

El sistema SHALL instalar las actualizaciones seleccionadas tras confirmación y MUST exigir permisos de administrador. Cuando la opción está marcada (por defecto), SHALL crear un punto de restauración antes y, si falla, preguntar si continuar. SHALL informar del resultado de cada controlador y de si hace falta reiniciar. MUST rechazar identificadores que no estén en la última búsqueda.

#### Scenario: Sin permisos
- **WHEN** la app no es administradora
- **THEN** la instalación está deshabilitada y el backend la rechaza

#### Scenario: Requiere reinicio
- **WHEN** un controlador instalado necesita reiniciar
- **THEN** se indica que hay que reiniciar el equipo

### Requirement: Herramientas oficiales de tarjetas gráficas

Para cada tarjeta gráfica de NVIDIA, AMD o Intel, el sistema SHALL ofrecer un botón que abre en el navegador la página oficial de controladores del fabricante. Solo se SHALL abrir direcciones de una lista fija.

#### Scenario: NVIDIA
- **WHEN** el usuario pulsa el botón junto a la tarjeta NVIDIA
- **THEN** se abre la página oficial de la app de NVIDIA

### Requirement: Registro de controladores instalados

El sistema SHALL añadir una línea a `%LOCALAPPDATA%\DonLimpiezas\drivers.log` por cada controlador procesado con fecha ISO 8601, título y resultado.

#### Scenario: Registro
- **WHEN** se instala un controlador de audio de Realtek
- **THEN** el log contiene una línea con la fecha, su título y "instalado"
