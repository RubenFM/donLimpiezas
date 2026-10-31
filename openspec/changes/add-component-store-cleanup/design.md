# Design

## Context

Ver `proposal.md`. `cleaner.rs` calcula tamaños recorriendo archivos; WinSxS no se puede medir así (enlaces duros compartidos con System32).

## Goals / Non-Goals

**Goals:**
- Integrarlo como una categoría más, con la misma confirmación y resumen.

**Non-Goals:**
- Analizar el almacén (`/AnalyzeComponentStore`), cuya salida además está traducida al idioma del sistema.

## Decisions

- Nuevo `Target::ComponentStore`. Al analizar suma 0 y marca `size_known = false`; la interfaz muestra "—" y "tamaño desconocido".
- Al limpiar: espacio libre de `%SystemDrive%\` antes y después (`GetDiskFreeSpaceExW`, ya usado en `sysinfo.rs`), y `Dism.exe /Online /Cleanup-Image /StartComponentCleanup` sin ventana. Liberado = diferencia (mínimo 0). Código de salida distinto de 0 = fallido.
- Categoría desmarcada por defecto con aviso, como las de Privacidad (`opt_in`).

## Risks / Trade-offs

- [Otros programas escriben en el disco mientras tanto] → La cifra es aproximada; se indica como "aprox."
- [Tarda minutos sin progreso] → Aviso previo y el botón queda en "Limpiando…".
