# Proposal

## Why

Los controladores antiguos causan fallos y pérdidas de rendimiento, pero las "apps de drivers" de terceros suelen instalar controladores de fuentes dudosas. Windows Update ya distribuye controladores verificados por Microsoft; solo falta hacerlo visible.

## What Changes

- Nuevo módulo **Driver Updater**, el último de la barra lateral.
- Inventario de controladores instalados (dispositivo, clase, proveedor, versión y fecha) con búsqueda.
- Buscar actualizaciones de controladores **solo en Windows Update** e instalar las seleccionadas, con punto de restauración previo (requiere administrador).
- Para tarjetas gráficas NVIDIA, AMD o Intel: botón a la herramienta oficial del fabricante. La app nunca descarga controladores por su cuenta.
- Registro en `%LOCALAPPDATA%\DonLimpiezas\drivers.log`.
- Fuera de alcance: desinstalar o revertir controladores y fuentes distintas de Windows Update.

## Capabilities

### New Capabilities
- `driver-updates`: inventario de controladores y actualización mediante Windows Update.

### Modified Capabilities

## Impact

- Nuevo `src-tauri/src/drivers.rs` y comandos `list_drivers`, `search_driver_updates`, `install_driver_updates` y `open_vendor_page`.
- El punto de restauración pasa de `debloat.rs` a `system.rs` para reutilizarlo con su propia descripción.
- Nuevo `src/modules/driver-updater.ts`.
