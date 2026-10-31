# Don Limpiezas

Limpiador de sistema y eliminador de bloatware para Windows 10/11, hecho con Tauri 2 (Rust + TypeScript).

## Funciones

**Custom Clean**
- Temporales del usuario y de Windows (solo archivos con más de 24 h)
- Papelera de reciclaje
- Caché de miniaturas, iconos y sombreadores DirectX; Prefetch y logs de CBS/DISM
- Volcados e informes de errores (usuario y sistema)
- Descargas de Windows Update
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

**Startup Manager**
- Lista los programas que se abren al iniciar sesión: claves `Run` del registro (usuario, equipo y
  equipo 32 bits) y carpetas de Inicio (usuario y común), con editor y destino de los accesos directos
- Los activa y desactiva igual que el Administrador de tareas (`StartupApproved`): no borra nada y
  ambos muestran siempre lo mismo
- Las entradas del equipo requieren administrador
- Guarda un registro en `%LOCALAPPDATA%\DonLimpiezas\startup.log`

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

## Estructura

```
src/
  main.ts             Arranque: barra lateral, aviso de permisos y módulo activo
  app.ts              Estado global (isAdmin)
  ui.ts               Utilidades compartidas (formato, diálogos, estados)
  modules/
    module.ts         Contrato de módulo e icon()
    index.ts          Lista ordenada de módulos de la barra lateral
    custom-clean.ts   Custom Clean
    uninstaller.ts    Uninstaller
    bloatware-remover.ts  Bloatware Remover
    startup-manager.ts    Startup Manager
src-tauri/src/
  lib.rs              Comandos expuestos a la interfaz
  cleaner.rs          Categorías de limpieza, análisis y borrado
  debloat.rs          Catálogo de bloatware y desinstalación vía PowerShell
  startup.rs          Programas de inicio y su estado en StartupApproved
  uninstaller.rs      Programas instalados y desinstalación con su desinstalador
  cmdline.rs          Interpretación de líneas de comandos registradas
  log.rs              Registro de cambios en %LOCALAPPDATA%\DonLimpiezas
  ps.rs               Ejecución de PowerShell sin ventana
  system.rs           Administrador, elevación y procesos en ejecución
```

Para añadir un módulo, crea `src/modules/<id>.ts` exportando un `Module` (id, nombre en inglés,
descripción en español, icono y `render(root)`, que busca sus elementos dentro de `root`) y
añádelo en su posición dentro de `src/modules/index.ts`.

Para añadir una categoría de limpieza, añade una entrada en `categories()` de `cleaner.rs`.
Para añadir una app al catálogo, añade una línea a `CATALOG` en `debloat.rs`.
