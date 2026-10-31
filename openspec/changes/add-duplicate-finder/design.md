# Design

## Context

Ver `proposal.md`. Hasta ahora todos los comandos devuelven un resultado al terminar; este análisis puede durar minutos, así que necesita progreso y cancelación.

## Goals / Non-Goals

**Goals:**
- Leer el mínimo de datos posible: la mayoría de archivos se descartan solo por tamaño.
- Imposible perder la última copia de un archivo.

**Non-Goals:**
- Persistir resultados entre sesiones.

## Decisions

### Algoritmo en tres pasadas

1. Recorrer las carpetas (sin seguir puntos de reanálisis; ignorando archivos con `FILE_ATTRIBUTE_OFFLINE` o `FILE_ATTRIBUTE_RECALL_ON_DATA_ACCESS`, que son marcadores de la nube) y agrupar por tamaño.
2. En los grupos de 2 o más, hash BLAKE3 de los primeros 64 KB.
3. En las coincidencias, hash BLAKE3 del archivo completo.

Las rutas se normalizan para no contar dos veces un archivo si se añaden carpetas anidadas. BLAKE3 se elige por velocidad; alternativa descartada: comparar byte a byte por parejas (cuadrático en grupos grandes).

### Progreso y cancelación

`find_duplicates` recibe el `AppHandle` y emite `duplicates-progress` con `{ phase, found, processed, total }` como mucho cada 100 ms. Un `AtomicBool` en el estado gestionado por Tauri se consulta en cada archivo; `cancel_duplicates` lo activa y el comando devuelve el error "Análisis cancelado".

### Validación al eliminar

El estado guarda los grupos del último análisis. `trash_duplicates(paths)` comprueba que cada ruta pertenece a un grupo y que en ningún grupo se seleccionan todas; si falla algo no se toca nada. Después envía todas las rutas a la papelera con `SHFileOperationW` (`FO_DELETE` + `FOF_ALLOWUNDO`, sin diálogos) y elimina del estado las que ya no existen.

### Carpetas por defecto

Con PowerShell: `[Environment]::GetFolderPath` para Documentos, Imágenes, Vídeos y Música, y `shell:Downloads` para Descargas, que respetan las redirecciones (OneDrive). Solo se devuelven las que existen.

### Selector de carpetas

`tauri-plugin-dialog` (`open({ directory: true, multiple: true })`) con el permiso `dialog:allow-open` en la capacidad por defecto.

## Risks / Trade-offs

- [Análisis largo en carpetas enormes] → Progreso visible, cancelación y tamaño mínimo por defecto de 1 MB.
- [Un archivo cambia entre el análisis y la eliminación] → Se comprueba que sigue existiendo; al ir a la papelera siempre es recuperable.
- [Muchos grupos ralentizan la interfaz] → Se muestran los 300 grupos con más espacio recuperable y se indica cuántos más hay.
