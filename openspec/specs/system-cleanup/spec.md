# system-cleanup Specification

## Purpose

Analizar y liberar espacio en disco borrando archivos temporales, cachés y restos del sistema que Windows y las aplicaciones regeneran solos, sin tocar datos del usuario.

Implementado en `src-tauri/src/cleaner.rs` (comandos `scan_cleanup` y `run_cleanup`) y en el módulo Custom Clean (`src/modules/custom-clean.ts`).

Categorías actuales (las de Privacidad van desmarcadas por defecto):

| id | Grupo | Admin | Origen |
|----|-------|-------|--------|
| `user_temp` | Windows | No | `%TEMP%`, más de 24 h |
| `windows_temp` | Windows | Sí | `%SystemRoot%\Temp`, más de 24 h |
| `recycle_bin` | Windows | No | Papelera de todas las unidades |
| `thumbnails` | Windows | No | `thumbcache_*` en `%LOCALAPPDATA%\Microsoft\Windows\Explorer` |
| `icon_cache` | Windows | No | `iconcache_*` en la misma carpeta |
| `shader_cache` | Windows | No | `%LOCALAPPDATA%\D3DSCache` |
| `prefetch` | Windows | Sí | `%SystemRoot%\Prefetch` |
| `windows_logs` | Windows | Sí | `%SystemRoot%\Logs\CBS` y `\DISM`, más de 24 h |
| `crash_dumps` | Windows | No | `%LOCALAPPDATA%\CrashDumps` y WER del usuario |
| `system_crash_dumps` | Windows | Sí | `%SystemRoot%\Minidump` y WER de `%ProgramData%` |
| `windows_update` | Windows | Sí | `%SystemRoot%\SoftwareDistribution\Download` |
| `chrome_cache` | Navegadores | No | `Cache`, `Code Cache`, `GPUCache` de cada perfil |
| `edge_cache` | Navegadores | No | Ídem para Edge |
| `brave_cache` | Navegadores | No | Ídem para Brave |
| `firefox_cache` | Navegadores | No | `cache2` de cada perfil |
| `discord_cache` | Aplicaciones | No | `%APPDATA%\discord\{Cache,Code Cache,GPUCache}` |
| `spotify_cache` | Aplicaciones | No | `%LOCALAPPDATA%\Spotify\Storage` |
| `vscode_cache` | Aplicaciones | No | `%APPDATA%\Code\{Cache,CachedData,Code Cache,GPUCache}` |
| `steam_cache` | Aplicaciones | No | `%LOCALAPPDATA%\Steam\htmlcache` |
| `chrome_privacy` | Privacidad | No | `History` y `Network\Cookies` de cada perfil |
| `edge_privacy` | Privacidad | No | Ídem para Edge |
| `brave_privacy` | Privacidad | No | Ídem para Brave |
| `firefox_cookies` | Privacidad | No | `cookies.sqlite` de cada perfil |
| `recent_docs` | Privacidad | No | `%APPDATA%\Microsoft\Windows\Recent` |

## Requirements

### Requirement: Categorías de limpieza

El sistema SHALL ofrecer las categorías de la tabla anterior, cada una con identificador, nombre, descripción, grupo e indicación de si requiere administrador. Las cachés de navegador SHALL cubrir todos los perfiles y MUST NOT incluir historial, contraseñas, cookies ni sesiones.

#### Scenario: Navegador con varios perfiles
- **WHEN** Chrome tiene los perfiles `Default` y `Profile 1`
- **THEN** la categoría `chrome_cache` incluye las carpetas de caché de ambos perfiles

#### Scenario: Navegador no instalado
- **WHEN** la carpeta de datos de un navegador no existe
- **THEN** su categoría se muestra con 0 bytes y 0 elementos

### Requirement: Análisis sin modificar nada

El sistema SHALL calcular, para cada categoría, el tamaño total en bytes y el número de elementos que se borrarían, sin modificar ningún archivo.

#### Scenario: Analizar
- **WHEN** el usuario pulsa "Analizar"
- **THEN** se muestra cada categoría agrupada con su tamaño y número de elementos
- **AND** ningún archivo del disco ha cambiado

### Requirement: Limpieza confirmada de categorías seleccionadas

El sistema SHALL borrar únicamente las categorías seleccionadas, tras una confirmación explícita del usuario que liste dichas categorías, y SHALL informar del espacio liberado, elementos borrados y elementos omitidos.

#### Scenario: Limpiar con confirmación
- **WHEN** el usuario selecciona categorías, pulsa "Limpiar seleccionados" y confirma
- **THEN** se borra el contenido de esas categorías
- **AND** se muestra el espacio liberado y los elementos borrados
- **AND** se vuelve a analizar automáticamente

#### Scenario: Cancelar la confirmación
- **WHEN** el usuario cancela el diálogo de confirmación
- **THEN** no se borra nada

### Requirement: Protección de archivos recientes de temporales

El sistema MUST ignorar en `user_temp` y `windows_temp` los archivos modificados en las últimas 24 horas, o cuya fecha no pueda leerse, tanto al analizar como al limpiar.

#### Scenario: Instalador en curso
- **WHEN** un instalador ha escrito archivos en `%TEMP%` hace 10 minutos
- **THEN** esos archivos no se cuentan ni se borran

### Requirement: Recorrido seguro del sistema de archivos

El sistema MUST NOT seguir enlaces simbólicos, uniones (junctions) ni otros puntos de reanálisis al recorrer carpetas, y MUST conservar la carpeta raíz de cada destino, borrando solo su contenido. Las subcarpetas solo se eliminan si quedan vacías.

#### Scenario: Unión dentro de una carpeta temporal
- **WHEN** `%TEMP%` contiene una unión que apunta a `C:\Users\ruben\Documents`
- **THEN** la unión no se recorre ni se borra su destino

### Requirement: Tolerancia a archivos en uso

El sistema SHALL continuar la limpieza cuando un archivo no pueda borrarse y SHALL contabilizarlo como omitido. Si el archivo es de solo lectura, SHALL quitar ese atributo y reintentar una vez.

#### Scenario: Archivo bloqueado
- **WHEN** un archivo de la caché está abierto por el navegador
- **THEN** se omite, el resto de la categoría se limpia y el resumen indica cuántos se omitieron

### Requirement: Papelera de reciclaje

El sistema SHALL obtener el tamaño y número de elementos de la papelera mediante `SHQueryRecycleBinW` y vaciarla mediante `SHEmptyRecycleBinW` sin diálogos, barra de progreso ni sonido.

#### Scenario: Vaciar la papelera
- **WHEN** se limpia la categoría `recycle_bin`
- **THEN** la papelera de todas las unidades queda vacía sin que Windows muestre ningún diálogo

### Requirement: Categorías que requieren administrador

El sistema SHALL mostrar las categorías que requieren administrador con una etiqueta "Admin" y SHALL deshabilitarlas y dejarlas sin seleccionar cuando la app no se ejecuta elevada.

#### Scenario: Sin permisos
- **WHEN** la app se ejecuta sin administrador
- **THEN** `windows_temp`, `system_crash_dumps` y `windows_update` aparecen deshabilitadas
