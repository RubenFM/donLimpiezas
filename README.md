# Don Limpiezas

Limpiador de sistema y eliminador de bloatware para Windows 10/11, hecho con Tauri 2 (Rust + TypeScript).

## Funciones

**Health Check**
- Resumen de solo lectura: Windows, tiempo encendido, memoria y espacio de cada disco
- Espacio liberable, bloatware, programas de inicio, actualizaciones y ajustes de privacidad
  pendientes, con un botón "Revisar" que abre el módulo correspondiente

**Custom Clean**
- Temporales del usuario y de Windows (solo archivos con más de 24 h)
- Papelera de reciclaje
- Caché de miniaturas, iconos y sombreadores DirectX; Prefetch y logs de CBS/DISM
- Volcados e informes de errores (usuario y sistema)
- Descargas de Windows Update y, desmarcada por defecto, limpieza de componentes antiguos con DISM
  (como administrador; tarda varios minutos)
- Caché de Chrome, Edge, Brave y Firefox (no toca historial, contraseñas ni sesiones)
- Caché de Discord, Spotify, Visual Studio Code y Steam
- Privacidad, desmarcada por defecto: historial y cookies de Chrome, Edge y Brave, cookies de
  Firefox y documentos recientes (nunca contraseñas ni marcadores)
- Avisa si un programa está abierto y conviene cerrarlo antes de limpiar

**Uninstaller**
- Lista los programas instalados (64 y 32 bits, equipo y usuario) con editor, versión, tamaño y fecha,
  sin componentes del sistema ni actualizaciones
- Búsqueda y orden por nombre, tamaño o fecha
- Desinstala varios seguidos con el desinstalador oficial de cada uno (con su asistente, nunca en
  silencio) y comprueba si se desinstalaron
- Guarda un registro en `%LOCALAPPDATA%\DonLimpiezas\uninstall.log`

**Bloatware Remover**
- Detecta el bloatware de un catálogo revisado (Noticias, El Tiempo, Candy Crush, Clipchamp, Xbox…)
- Lo quita para tu usuario o para todos los usuarios y la imagen de Windows (requiere administrador)
- Crea un punto de restauración antes de quitar nada
- Guarda un registro en `%LOCALAPPDATA%\DonLimpiezas\debloat.log`

Nunca se tocan Microsoft Store, los runtimes (VCLibs, .NET, UI.Xaml), el shell ni Seguridad de Windows.

**Disk Analyzer**
- Analiza un disco o una carpeta y muestra qué ocupa espacio, con barras y porcentajes
- Se navega entrando en carpetas y con la ruta de migas, sin volver a analizar; abre la carpeta en el
  Explorador. No borra nada. Los archivos solo en la nube cuentan 0 bytes

**Duplicate Finder**
- Busca archivos con el mismo contenido (hash BLAKE3) en Documentos, Descargas, Imágenes, Vídeos y
  Música, o en las carpetas que elijas, con tamaño mínimo, progreso y cancelación
- Ignora los archivos de OneDrive que solo están en la nube, para no descargarlos
- "Conservar el más antiguo" selecciona el resto de copias; nunca deja borrar todas las de un grupo
- Envía los archivos a la papelera y lo registra en `%LOCALAPPDATA%\DonLimpiezas\duplicates.log`

**Cloud Drive Cleaner**
- Detecta OneDrive (todas las cuentas) y Dropbox y lista los archivos descargados en el equipo por
  tamaño y antigüedad, sin abrirlos ni descargar nada
- "Liberar espacio" los deja solo en la nube (`attrib -P +U`, como el Explorador): no se borran y se
  descargan al abrirlos. Registro en `%LOCALAPPDATA%\DonLimpiezas\cloud.log`

**Performance Optimizer**
- Catálogo de ajustes de registro revisados: telemetría, anuncios y sugerencias, Widgets, Copilot,
  búsqueda web del menú Inicio y grabación de juegos en segundo plano
- Muestra cuáles están ya aplicados; los aplica guardando antes el valor original en
  `%LOCALAPPDATA%\DonLimpiezas\tweaks.json` y los restaura exactamente, uno a uno o todos
- Guarda un registro en `%LOCALAPPDATA%\DonLimpiezas\tweaks.log`

**Startup Manager**
- Lista los programas que se abren al iniciar sesión: claves `Run` del registro (usuario, equipo y
  equipo 32 bits) y carpetas de Inicio (usuario y común), con editor y destino de los accesos directos
- También tareas programadas de terceros que se lanzan al iniciar sesión o arrancar, y servicios
  automáticos de terceros (al desactivarlos pasan a inicio manual, nunca a deshabilitados). Excluye
  siempre lo de Windows y Microsoft Defender
- Los activa y desactiva igual que el Administrador de tareas (`StartupApproved`): no borra nada y
  ambos muestran siempre lo mismo
- Las entradas del equipo requieren administrador
- Guarda un registro en `%LOCALAPPDATA%\DonLimpiezas\startup.log`

**Software Updater**
- Lista los programas con versión nueva según winget (también los de versión desconocida)
- Los actualiza de uno en uno con sus instaladores oficiales y lo registra en
  `%LOCALAPPDATA%\DonLimpiezas\updates.log`
- Al abrir la app comprueba las actualizaciones y muestra cuántas hay en la barra lateral

**Driver Updater**
- Inventario de controladores con filtro
- Busca e instala actualizaciones de controladores solo con Windows Update (instalar requiere
  administrador), con punto de restauración previo. Registro en `%LOCALAPPDATA%\DonLimpiezas\drivers.log`
- Para tarjetas NVIDIA, AMD o Intel, botón a la herramienta oficial del fabricante

**History**
- Todo lo que han cambiado los módulos (a partir de sus logs), agrupado por día y filtrable
- Deshacer cuando sigue siendo válido: reactivar o desactivar un programa de inicio, restaurar o
  volver a aplicar un ajuste, buscar en Microsoft Store una app quitada, abrir la papelera o volver
  a descargar un archivo liberado de la nube

## Requisitos (en Windows)

1. [Node.js](https://nodejs.org) (ya lo tienes)
2. [Rust](https://rustup.rs): descarga y ejecuta `rustup-init.exe`
3. [Visual Studio Build Tools](https://visualstudio.microsoft.com/visual-cpp-build-tools/) con la carga de trabajo **"Desarrollo para el escritorio con C++"**
4. WebView2 (ya viene con Windows 11)

## Uso

Desde PowerShell, en esta carpeta:

```powershell
npm install
npm run tauri dev     # modo desarrollo
npm run tauri build   # instalador en src-tauri\target\release\bundle\
cd src-tauri; cargo test                       # tests unitarios
cd src-tauri; cargo test -- --ignored --nocapture  # tests que leen el sistema real
```

Para probar las funciones que necesitan administrador en modo desarrollo, abre la terminal
**como administrador**. El botón "Reiniciar como administrador" está pensado para la app ya instalada.

## Icono

El icono (una fregona) está en `src-tauri/icons/app-icon.svg`. Para regenerar los tamaños después de
editarlo, exporta el SVG a `app-icon.png` (1024×1024, fondo transparente) y ejecuta
`npx tauri icon src-tauri\icons\app-icon.png`; borra después las carpetas `android` e `ios` y los
`Square*`, que no se usan en Windows. La barra lateral usa una copia en `src/assets/logo.svg`.

## Estructura

```
src/
  main.ts             Arranque: barra lateral, aviso de permisos y módulo activo
  app.ts              Estado global (isAdmin)
  ui.ts               Utilidades compartidas (formato, diálogos, estados)
  nav.ts              Abrir módulos y contadores de la barra lateral desde un módulo
  updates.ts          Lista de actualizaciones compartida entre módulos
  modules/
    module.ts         Contrato de módulo e icon()
    index.ts          Lista ordenada de módulos de la barra lateral
    health-check.ts   Health Check
    custom-clean.ts   Custom Clean
    uninstaller.ts    Uninstaller
    bloatware-remover.ts  Bloatware Remover
    disk-analyzer.ts      Disk Analyzer
    duplicate-finder.ts   Duplicate Finder
    cloud-drive-cleaner.ts  Cloud Drive Cleaner
    performance-optimizer.ts  Performance Optimizer
    startup-manager.ts    Startup Manager
    software-updater.ts   Software Updater
    driver-updater.ts     Driver Updater
    history.ts            History
src-tauri/src/
  lib.rs              Comandos expuestos a la interfaz
  cleaner.rs          Categorías de limpieza, análisis y borrado
  debloat.rs          Catálogo de bloatware y desinstalación vía PowerShell
  diskusage.rs        Tamaño por carpetas para Disk Analyzer
  duplicates.rs       Búsqueda de duplicados por contenido y papelera
  tweaks.rs           Catálogo de ajustes reversibles y valores originales
  cloud.rs            Carpetas de OneDrive/Dropbox y liberar espacio local
  drivers.rs          Inventario de controladores y Windows Update
  startup.rs          Programas de inicio y su estado en StartupApproved
  uninstaller.rs      Programas instalados y desinstalación con su desinstalador
  cmdline.rs          Interpretación de líneas de comandos registradas
  updater.rs          Lectura de winget y actualización de programas
  sysinfo.rs          Windows, memoria, discos y tiempo encendido
  log.rs              Registro de cambios en %LOCALAPPDATA%\DonLimpiezas
  history.rs          Lectura de todos los logs y deshacer
  ps.rs               Ejecución de PowerShell sin ventana
  system.rs           Administrador, elevación, puntos de restauración y procesos
```

Para añadir un módulo, crea `src/modules/<id>.ts` exportando un `Module` (id, nombre en inglés,
descripción en español, icono, `render(root)`, que busca sus elementos dentro de `root`, e `init()` opcional al arrancar) y
añádelo en su posición dentro de `src/modules/index.ts`.

Para añadir una categoría de limpieza, añade una entrada en `categories()` de `cleaner.rs`.
Para añadir una app al catálogo, añade una línea a `CATALOG` en `debloat.rs`.
