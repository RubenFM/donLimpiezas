# Design

## Context

Ver `proposal.md`. En el equipo de desarrollo, la búsqueda de controladores con el agente de Windows Update (`Microsoft.Update.Session`) funciona sin administrador y tarda unos 8 s; el inventario (`Win32_PnPSignedDriver`) devuelve 238 controladores.

## Goals / Non-Goals

**Goals:**
- Ningún binario de controlador pasa por la app: todo lo descarga e instala Windows Update.

**Non-Goals:**
- Comparar versiones con las webs de los fabricantes.

## Decisions

### Inventario

PowerShell: `Get-CimInstance Win32_PnPSignedDriver` filtrado a entradas con nombre y versión; se devuelve nombre, clase, proveedor, versión y fecha (`yyyy-MM-dd`) en JSON. Se ordena por clase y nombre.

### Buscar e instalar con el agente de Windows Update

- Buscar: `CreateUpdateSearcher().Search("IsInstalled=0 and Type='Driver' and IsHidden=0")`; se devuelve `UpdateID`, título, clase, fabricante, fecha y `MaxDownloadSize`. Se guarda la lista en un estado de Tauri.
- Instalar: se valida que cada id esté en la última búsqueda y tenga formato GUID (así es seguro incrustarlo en el script); el script vuelve a buscar, añade a una `UpdateColl` los seleccionados (aceptando su EULA), descarga e instala, y devuelve por cada uno `ResultCode` (2 = correcto, 3 = correcto con errores) y `RebootRequired`.

### Punto de restauración compartido

`create_restore_point` se mueve a `system.rs` con un parámetro de descripción. Bloatware Remover y Driver Updater lo usan con su propio texto. El comando de Bloatware Remover mantiene su nombre.

### Páginas de los fabricantes

`open_vendor_page(vendor)` acepta solo `nvidia`, `amd` o `intel` y abre con `ShellExecuteW` una URL fija:

| Fabricante | URL |
|---|---|
| NVIDIA | `https://www.nvidia.com/es-es/software/nvidia-app/` |
| AMD | `https://www.amd.com/es/support/download/drivers.html` |
| Intel | `https://www.intel.es/content/www/es/es/support/detect.html` |

El fabricante se deduce del proveedor del controlador de clase `DISPLAY`.

## Risks / Trade-offs

- [Un controlador de Windows Update da problemas] → Punto de restauración previo y solo fuente Microsoft.
- [La instalación tarda varios minutos] → Mensaje de progreso y el comando corre en otro hilo.
- [El agente de Windows Update está deshabilitado por política] → Se muestra el error de la búsqueda tal cual.
