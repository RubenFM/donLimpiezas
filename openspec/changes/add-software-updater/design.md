# Design

## Context

Ver `proposal.md`. `winget upgrade` no tiene salida JSON en la versión instalada (1.29) y el módulo `Microsoft.WinGet.Client` no viene con Windows.

## Goals / Non-Goals

**Goals:**
- Leer la tabla de winget de forma independiente del idioma.
- No ejecutar winget dos veces cuando varios módulos necesitan la lista.

**Non-Goals:**
- Instalar el módulo de PowerShell de winget.

## Decisions

### Lectura de la tabla por columnas

Se ejecuta `winget upgrade --include-unknown --accept-source-agreements --disable-interactivity` sin ventana. De cada línea se toma lo que hay tras el último `\r` (winget dibuja un indicador de progreso). La línea de cabecera es la anterior a la primera línea formada solo por guiones; cada columna empieza donde empieza una palabra de la cabecera, así que no importa el idioma. Las filas terminan en la primera línea vacía o demasiado corta (el pie "N actualizaciones disponibles", o la segunda tabla de paquetes que requieren selección explícita). Se descartan filas sin identificador, con espacios en él o truncadas con `…`.

### Actualizar

`winget upgrade --id <id> --exact --include-unknown --accept-package-agreements --accept-source-agreements --disable-interactivity`, sin `--silent`: el instalador muestra su interfaz si la tiene y winget pide UAC si hace falta. Código de salida 0 = actualizado; si no, se devuelve la última línea útil de la salida.

La última lista se guarda en un estado gestionado por Tauri para validar los ids.

### Contador y arranque

El contrato de módulo gana `init?()`, que `main.ts` llama al arrancar para todos los módulos. Un nuevo `src/nav.ts` expone `setBadge(id, n)` y `navigate(id)`, implementados por `main.ts` (evita importaciones circulares). `src/updates.ts` guarda la promesa de la última consulta, así que el contador, el módulo y Health Check comparten una sola ejecución de winget.

## Risks / Trade-offs

- [Un cambio de formato en futuras versiones de winget] → Tests unitarios con salidas reales en inglés y español; si no se reconoce la tabla, la lista sale vacía con un aviso en vez de datos erróneos.
- [Nombres con caracteres anchos desalinean columnas] → Solo afecta a la columna de nombre visible; el id se valida (sin espacios) antes de aceptarse.
