# Design

## Context

Ver `proposal.md`. `cleaner.rs` define cada categoría con destinos `Contents` (contenido de una carpeta), `FilesWithPrefix` y `RecycleBin`. La interfaz marca por defecto toda categoría con bytes y sin bloqueo de admin.

## Goals / Non-Goals

**Goals:**
- Añadir categorías sin cambiar el modelo de recorrido seguro existente.
- Que la selección por defecto y los avisos los decida el backend, no la interfaz.

**Non-Goals:**
- Editar bases de datos SQLite de los navegadores (borrar entradas concretas). Se borran archivos completos, que el navegador recrea vacíos.

## Decisions

### Nuevos campos de categoría

- `default_selected: bool`: `false` en Privacidad. La interfaz marca una categoría si `defaultSelected && bytes > 0 && !bloqueada`.
- `warning: Option<&str>`: texto fijo que la interfaz muestra bajo la descripción (p. ej. "Cerrará tu sesión en las webs").
- `processes: &[&str]`: ejecutables asociados. El análisis devuelve `running: bool` si alguno está en marcha.

### Destino `Files(Vec<PathBuf>)`

Para historial y cookies hay que borrar archivos concretos de cada perfil, no carpetas enteras: `History`, `History-journal`, `Network\Cookies`, `Network\Cookies-journal` (Chromium) y `cookies.sqlite`, `cookies.sqlite-wal` (Firefox). Reutiliza `handle_file`, así que hereda el reintento de solo lectura y el conteo de fallos.

Los perfiles de Chromium se enumeran igual que para la caché (`chromium_caches` pasa a un helper `chromium_profiles`).

### Procesos en ejecución con ToolHelp

`CreateToolhelp32Snapshot` + `Process32FirstW/NextW` de `windows-sys` da los nombres de ejecutable en una sola llamada barata, sin PowerShell. Se compara en minúsculas. Alternativa descartada: `Get-Process` vía PowerShell (un proceso extra en cada análisis).

### Categorías y rutas

| id | Grupo | Admin | Procesos | Destino |
|---|---|---|---|---|
| `icon_cache` | Windows | No | — | `iconcache_*` en `%LOCALAPPDATA%\Microsoft\Windows\Explorer` |
| `prefetch` | Windows | Sí | — | `%SystemRoot%\Prefetch` |
| `windows_logs` | Windows | Sí | — | `%SystemRoot%\Logs\CBS` y `\DISM`, más de 24 h |
| `discord_cache` | Aplicaciones | No | `discord.exe` | `%APPDATA%\discord\{Cache,Code Cache,GPUCache}` |
| `spotify_cache` | Aplicaciones | No | `spotify.exe` | `%LOCALAPPDATA%\Spotify\Storage` |
| `vscode_cache` | Aplicaciones | No | `code.exe` | `%APPDATA%\Code\{Cache,CachedData,Code Cache,GPUCache}` |
| `steam_cache` | Aplicaciones | No | `steam.exe`, `steamwebhelper.exe` | `%LOCALAPPDATA%\Steam\htmlcache` |
| `chrome_privacy` | Privacidad | No | `chrome.exe` | Historial y cookies de cada perfil |
| `edge_privacy` | Privacidad | No | `msedge.exe` | Ídem |
| `brave_privacy` | Privacidad | No | `brave.exe` | Ídem |
| `firefox_cookies` | Privacidad | No | `firefox.exe` | `cookies.sqlite*` de cada perfil |
| `recent_docs` | Privacidad | No | — | `%APPDATA%\Microsoft\Windows\Recent` |

Las cachés de navegador existentes añaden su proceso (`chrome.exe`, `msedge.exe`, `brave.exe`, `firefox.exe`).

## Risks / Trade-offs

- [Edge suele quedarse en segundo plano y aparecerá casi siempre como "Abierto"] → Es correcto: sus archivos pueden estar en uso. El aviso no bloquea.
- [Borrar Prefetch ralentiza un poco el primer arranque de cada programa] → Se indica en la descripción.
- [Rutas de apps de terceros cambian entre versiones] → Si la carpeta no existe, la categoría queda a 0 sin errores.
