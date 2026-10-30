# Proposal

> **Estado: borrador.** Pendiente de revisar antes de generar specs, design y tasks.

## Why

Los controladores antiguos causan fallos y pérdida de rendimiento, pero las "apps de drivers" de terceros suelen instalar controladores de fuentes dudosas.

## What Changes

- Inventario de controladores (dispositivo, fabricante, versión, fecha) vía `Win32_PnPSignedDriver`.
- Buscar actualizaciones **solo a través de Windows Update** (API `Microsoft.Update.Session`, categoría Drivers) e instalarlas desde ahí.
- Para GPUs (NVIDIA, AMD, Intel): enlace a la herramienta oficial del fabricante, sin descargar controladores por nuestra cuenta.
- Punto de restauración antes de instalar.

## Capabilities

### New Capabilities
- `driver-updates`: inventario de controladores y actualización mediante Windows Update.

### Modified Capabilities

## Impact

- Nuevo `src-tauri/src/drivers.rs` (PowerShell + COM de Windows Update); requiere administrador.
- Riesgo alto: un controlador defectuoso puede dejar el equipo sin arrancar. Por eso solo fuentes de Microsoft y siempre con punto de restauración.
