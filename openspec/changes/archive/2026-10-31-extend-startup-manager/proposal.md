# Proposal

## Why

Startup Manager solo cubre las claves `Run` y las carpetas de Inicio. Muchos fabricantes (GIGABYTE, Dell, SanDisk, NVIDIA…) arrancan sus programas con tareas programadas al iniciar sesión o con servicios automáticos, que no se ven ni en la app ni en la pestaña Inicio del Administrador de tareas.

## What Changes

- Startup Manager muestra también:
  - **Tareas programadas** de terceros que se ejecutan al iniciar sesión o al arrancar el equipo, con su ejecutable y autor. Se activan o desactivan como en el Programador de tareas.
  - **Servicios** de terceros con inicio automático. "Desactivar" los pasa a **inicio manual** (no los deshabilita): se inician solos cuando un programa los necesita. Surte efecto al reiniciar.
- Se excluyen siempre los elementos de Windows (ejecutable dentro de la carpeta de Windows o tareas de `\Microsoft\`) y los de Microsoft Defender.
- Ambos requieren administrador y se registran en `startup.log`, así que aparecen en History y se pueden deshacer desde allí.
- Fuera de alcance: deshabilitar servicios por completo, servicios de Windows y tareas con otros disparadores.

## Capabilities

### New Capabilities

### Modified Capabilities
- `startup-programs`: nuevos orígenes de tareas programadas y servicios.

## Impact

- `src-tauri/src/startup.rs`: orígenes `ScheduledTask` y `Service`, listado con una única llamada a PowerShell y `services.json` con los servicios pasados a manual por la app.
- `src/modules/startup-manager.ts`: notas por tipo de origen.
