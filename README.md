# Don Limpiezas

Limpiador de sistema y eliminador de bloatware para Windows 10/11, hecho con Tauri 2 (Rust + TypeScript).

## Funciones

**Limpieza**
- Temporales del usuario y de Windows (solo archivos con más de 24 h)
- Papelera de reciclaje
- Caché de miniaturas y de sombreadores DirectX
- Volcados e informes de errores (usuario y sistema)
- Descargas de Windows Update
- Caché de Chrome, Edge, Brave y Firefox (no toca historial, contraseñas ni sesiones)

**Apps preinstaladas**
- Detecta el bloatware de un catálogo revisado (Noticias, El Tiempo, Candy Crush, Clipchamp, Xbox…)
- Lo quita para tu usuario o para todos los usuarios y la imagen de Windows (requiere administrador)
- Crea un punto de restauración antes de quitar nada
- Guarda un registro en `%LOCALAPPDATA%\DonLimpiezas\debloat.log`

Nunca se tocan Microsoft Store, los runtimes (VCLibs, .NET, UI.Xaml), el shell ni Seguridad de Windows.

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
```

Para probar las funciones que necesitan administrador en modo desarrollo, abre la terminal
**como administrador**. El botón "Reiniciar como administrador" está pensado para la app ya instalada.

## Estructura

```
src/                  Interfaz (TypeScript + CSS)
src-tauri/src/
  lib.rs              Comandos expuestos a la interfaz
  cleaner.rs          Categorías de limpieza, análisis y borrado
  debloat.rs          Catálogo de bloatware y desinstalación vía PowerShell
  ps.rs               Ejecución de PowerShell sin ventana
  system.rs           Detección y elevación de administrador
```

Para añadir una categoría de limpieza, añade una entrada en `categories()` de `cleaner.rs`.
Para añadir una app al catálogo, añade una línea a `CATALOG` en `debloat.rs`.
