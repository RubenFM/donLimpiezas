# Design

## Context

Ver `proposal.md`. Los programas Win32 se registran en `Software\Microsoft\Windows\CurrentVersion\Uninstall\<clave>` con valores como `DisplayName`, `UninstallString`, `EstimatedSize` (KB, DWORD) e `InstallDate` (`yyyymmdd`). Ya usamos `winreg` en `startup.rs`.

## Goals / Non-Goals

**Goals:**
- Lanzar el desinstalador exactamente como lo haría Configuración, incluida la elevación.
- No bloquear la interfaz mientras un desinstalador está abierto.

**Non-Goals:**
- Detectar procesos hijos de desinstaladores que se relanzan a sí mismos (p. ej. copiándose a `%TEMP%`): en ese caso se informa que "sigue instalado" y el usuario puede actualizar la lista al terminar.

## Decisions

### Identificador de programa

`<vista>\<nombre de subclave>`, con vista `hklm64`, `hklm32` o `hkcu`. Es estable mientras el programa esté instalado y permite volver a localizar la clave para validar y comprobar el resultado.

### Filtrado

Se omite una entrada si: no tiene `DisplayName` o `UninstallString`; `SystemComponent = 1`; tiene `ParentKeyName`; o `ReleaseType` es `Update`, `Hotfix` o `Security Update`.

### Lanzamiento con `ShellExecuteExW`

`std::process::Command` usa `CreateProcess`, que falla con el error 740 si el ejecutable pide elevación y la app no es administradora. `ShellExecuteExW` (verbo por defecto, `SEE_MASK_NOCLOSEPROCESS`) muestra el UAC igual que el Explorador y devuelve el proceso para esperar con `WaitForSingleObject`. Alternativa descartada: `Start-Process` de PowerShell (otro proceso intermedio y peor control de errores).

### Línea de comandos compartida

`extract_exe` y `expand_env` de `startup.rs` pasan a `cmdline.rs` como `split(command) -> (exe, args)`. Para MSI se detecta `msiexec` y se reescribe a `msiexec.exe /X{GUID}` con el GUID del propio comando o, si no lo hay, el nombre de la subclave.

### Cola en la interfaz

La interfaz llama a `uninstall_program` de uno en uno y muestra "Desinstalando X (2 de 3)…". Cada llamada es bloqueante en un hilo de `spawn_blocking`, así que la ventana sigue respondiendo.

### Módulo

`Uninstaller` con búsqueda, selector de orden, casillas y "Desinstalar seleccionados". Icono de papelera.

## Risks / Trade-offs

- [Algunos desinstaladores devuelven el control enseguida y siguen en segundo plano] → Se informa "sigue instalado"; al pulsar "Actualizar" se ve el estado real.
- [`EstimatedSize` falta o es inexacto en muchos programas] → Se muestra "—" y esos programas van al final al ordenar por tamaño.
- [El usuario desinstala algo necesario] → Siempre con el asistente oficial, que puede cancelar, y confirmación previa.
