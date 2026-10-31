# Design

## Context

Ver `proposal.md`. Los análisis ya existen como comandos de solo lectura. Falta información básica del sistema y una forma de abrir un módulo desde otro (`navigate` de `src/nav.ts`, introducido en `add-software-updater`).

## Goals / Non-Goals

**Goals:**
- Reutilizar los comandos existentes sin duplicar lógica.

**Non-Goals:**
- Cachear análisis entre módulos (salvo las actualizaciones, ya compartidas).

## Decisions

### Información del sistema en Rust

- Windows: `ProductName`, `DisplayVersion` y `CurrentBuild` de `HKLM\SOFTWARE\Microsoft\Windows NT\CurrentVersion`; si la compilación es ≥ 22000 se sustituye "Windows 10" por "Windows 11" (el registro no lo actualiza).
- Tiempo encendido: `GetTickCount64`.
- Memoria: `GlobalMemoryStatusEx`.
- Discos: `GetLogicalDriveStringsW`, solo `DRIVE_FIXED`, con `GetDiskFreeSpaceExW`.

Alternativa descartada: WMI vía PowerShell (más lento y otro proceso).

### Apartados

| Apartado | Comando | Valor | Mejorable si |
|---|---|---|---|
| Espacio liberable | `scan_cleanup` | Suma de categorías marcadas por defecto y no bloqueadas | ≥ 500 MB |
| Bloatware | `list_bloat` | Apps del catálogo instaladas | ≥ 1 |
| Programas de inicio | `list_startup` | Entradas activadas | ≥ 6 |
| Actualizaciones | caché de `updates.ts` | Disponibles | ≥ 1 |
| Privacidad | `list_tweaks` | Ajustes sin aplicar disponibles sin admin o con admin si lo es | ≥ 1 |

Cada apartado es una promesa independiente que pinta su tarjeta al resolverse.

## Risks / Trade-offs

- [Abrir Health Check lanza varios análisis a la vez] → Son de solo lectura y en hilos aparte; la interfaz muestra cada tarjeta al terminar.
- [Los umbrales de "mejorable" son arbitrarios] → Son constantes en un solo sitio y fáciles de ajustar.
