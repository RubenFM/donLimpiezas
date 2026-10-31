# Design

## Context

Ver `proposal.md`. Un disco de 1,8 TB puede tener millones de archivos; guardar cada archivo en memoria no es viable, y recorrer de nuevo al navegar sería lento.

## Goals / Non-Goals

**Goals:**
- Un único recorrido; navegación instantánea después.
- Memoria acotada aunque haya millones de archivos.

**Non-Goals:**
- Tamaño en disco por clústeres o compresión: se usa el tamaño lógico (0 si está solo en la nube).

## Decisions

### Modelo en memoria

Por cada carpeta: tamaño total, número de archivos, subcarpetas (nombre y tamaño), los 20 archivos más grandes y el total del resto. Se guarda en un `HashMap<ruta en minúsculas, Nodo>` en el estado de Tauri. `disk_children(ruta)` lo lee sin tocar el disco.

### Recorrido

Iterativo en profundidad con pila y cálculo de tamaños en post-orden. Se usan los metadatos de `DirEntry` (en Windows vienen de `FindNextFileW`, sin abrir cada archivo). Se descartan enlaces y uniones (`is_symlink`, que en Windows cubre los sustitutos de nombre) pero no otros puntos de reanálisis (OneDrive). Archivos con atributos de "solo en la nube" suman 0. Las carpetas ilegibles se cuentan.

Progreso por evento `disk-progress` como mucho cada 150 ms; cancelación con `AtomicBool`, como en Duplicate Finder.

### Abrir en el Explorador

`ShellExecuteExW` con el verbo `explore` y la ruta, solo si está en el último análisis.

### Interfaz

Selector con los discos fijos (de `system_info`) y "Elegir carpeta…" (diálogo). Ruta de migas, lista con barras y porcentajes (carpetas clicables) y resumen (total, archivos, carpetas ilegibles, duración).

## Risks / Trade-offs

- [Análisis largo de un disco entero] → Progreso, cancelación y la posibilidad de analizar solo una carpeta.
- [Memoria con cientos de miles de carpetas] → Solo carpetas y 20 archivos por carpeta; del orden de decenas de MB en el peor caso.
