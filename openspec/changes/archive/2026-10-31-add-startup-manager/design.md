# Design

## Context

Ver `proposal.md` (Why) y `specs/startup-programs/spec.md` (requisitos).

Windows (8 y posteriores) guarda el estado activado/desactivado de los programas de inicio por separado de las entradas, en `Software\Microsoft\Windows\CurrentVersion\Explorer\StartupApproved\{Run,Run32,StartupFolder}` bajo `HKCU` o `HKLM`. Es lo que lee y escribe el Administrador de tareas. Cada valor tiene el mismo nombre que la entrada y es un `REG_BINARY` de 12 bytes: el primer byte par (`0x02`, `0x06`) significa activado e impar (`0x03`, `0x07`) desactivado; los bytes 4–11 guardan el `FILETIME` en que se desactivó.

## Goals / Non-Goals

**Goals:**
- Reutilizar el mecanismo de `StartupApproved` para que Don Limpiezas y el Administrador de tareas muestren siempre lo mismo.
- Un único módulo backend sin dependencias de PowerShell para lo crítico (leer y escribir el registro).

**Non-Goals:**
- Resolver argumentos complejos de la línea de comandos ni verificar firmas digitales.
- Vigilar cambios en tiempo real: la lista se refresca al abrir el módulo por primera vez o al pulsar "Actualizar".

## Decisions

### Orígenes y su clave de aprobación

| id de origen | Entradas | Aprobación |
|---|---|---|
| `hkcu_run` | `HKCU\…\Run` | `HKCU\…\StartupApproved\Run` |
| `hklm_run` | `HKLM\…\Run` (vista 64 bits) | `HKLM\…\StartupApproved\Run` |
| `hklm_run32` | `HKLM\…\Run` (vista 32 bits, `KEY_WOW64_32KEY`) | `HKLM\…\StartupApproved\Run32` |
| `user_folder` | `%APPDATA%\Microsoft\Windows\Start Menu\Programs\Startup` | `HKCU\…\StartupApproved\StartupFolder` |
| `common_folder` | `%ProgramData%\Microsoft\Windows\Start Menu\Programs\Startup` | `HKLM\…\StartupApproved\StartupFolder` |

En las carpetas, el nombre del valor de aprobación es el nombre del archivo con extensión (`Discord.lnk`); se ignora `desktop.ini`. La identidad de una entrada es `(origen, nombre)`.

### Escritura del estado

- Desactivar: `03 00 00 00` + `FILETIME` actual (8 bytes, little-endian).
- Activar: `02 00 00 00` + 8 bytes a cero.
- Antes de escribir se vuelve a listar el origen y se comprueba que la entrada existe (requisito "Solo entradas detectadas"). Los orígenes `hklm_*` y `common_folder` comprueban `is_admin()` en el backend, no solo en la interfaz.

### Acceso al registro: crate `winreg`

Alternativas:
- `windows-sys` con `Win32_System_Registry`: sin dependencias nuevas, pero mucho código `unsafe` para manejar buffers y vistas WOW64.
- PowerShell: lento (un proceso por llamada) y torpe con `REG_BINARY`.

Se elige `winreg` (API segura, soporta `KEY_WOW64_32KEY` y valores binarios). Es una dependencia pequeña y muy usada.

### Destino de accesos directos y editor: una llamada a PowerShell

Resolver `.lnk` (COM `IShellLink`) y leer `CompanyName` de un ejecutable desde Rust requiere bastante código COM y de recursos de versión. Se hace con **una sola** llamada a `ps::run` por listado, que recibe todas las rutas (escapadas con `ps::quote`) y devuelve JSON con `{ path, target, company }`. Si falla, el listado se muestra igualmente sin destino ni editor.

La ruta del ejecutable de una entrada `Run` se extrae del comando: el primer token entre comillas, o hasta el primer `.exe` si no las hay; las variables de entorno (`%ProgramFiles%`) se expanden.

### Registro de cambios con `chrono`

`debloat.log` usa `Get-Date` desde PowerShell; aquí no hay script, así que se añade `chrono` (solo la feature `clock`) para la fecha ISO 8601 local. Se crea un helper `log::append(file, line)` reutilizable por futuros módulos.

### Interfaz

Nuevo módulo "Startup Manager" con una fila por entrada e interruptor. El cambio se aplica al pulsar el interruptor, **sin diálogo de confirmación**: no es destructivo y se deshace con otro clic. Si el backend devuelve error, el interruptor vuelve a su estado anterior y se muestra el mensaje.

## Risks / Trade-offs

- [Algunos programas se vuelven a activar solos al actualizarse o reescriben su entrada `Run`] → Fuera de nuestro control; se refleja correctamente al refrescar porque leemos el estado real.
- [Formato de `StartupApproved` no documentado oficialmente] → Es estable desde Windows 8 y lo usan herramientas como Autoruns; se trata como desactivado solo un primer byte impar y se conserva el resto de bytes desconocidos al leer.
- [Un comando de `Run` mal formado impide extraer el ejecutable] → Se muestra el comando tal cual, sin editor.
- [Escribir en `HKLM` por error] → Comprobación de admin en backend y validación `(origen, nombre)` antes de cada escritura.
