# Design

## Context

Ver `proposal.md`. Formatos actuales (todas las líneas empiezan con fecha ISO 8601 y tabulador):

| Log | Campos tras la fecha |
|---|---|
| `debloat.log` | paquete, `OK` o `ERROR …` |
| `startup.log` | etiqueta de origen, nombre, `activado`/`desactivado` |
| `tweaks.log` | id, `aplicado`/`restaurado`/`ERROR …` |
| `uninstall.log` | nombre, versión, `desinstalado`/`sigue instalado` |
| `duplicates.log` | ruta |
| `cloud.log` | ruta (nuevo: ruta, `liberado`/`descargado`) |
| `updates.log` | id, `versión → versión`, `actualizado`/`ERROR …` |
| `drivers.log` | título, `instalado`/`ERROR …` |

## Goals / Non-Goals

**Goals:**
- No cambiar cómo registran los módulos (salvo la columna de acción en `cloud.log`).
- Que deshacer nunca dependa de datos de la interfaz.

**Non-Goals:**
- Un formato de log nuevo y común.

## Decisions

### Identificador de entrada

`<log>:<número de línea>`. `undo_history(id)` vuelve a leer esa línea, la interpreta y calcula la acción. Así la interfaz solo envía un identificador.

### Validez del deshacer

| Origen | Acción ofrecida | Válida si |
|---|---|---|
| Inicio desactivado/activado | Volver a activar/desactivar | Estado actual igual al registrado y es la última línea de esa entrada |
| Ajuste aplicado | Restaurar | Hay original guardado |
| Ajuste restaurado | Volver a aplicar | No está aplicado |
| App quitada (`OK`) | Buscar en Microsoft Store | Paquete del catálogo |
| Duplicado | Abrir la papelera | Siempre |
| Nube liberado | Volver a descargar | Archivo existe y no está en local |

Los orígenes de inicio se registran por etiqueta; `Source::from_label` los recupera. Los ajustes que requieren administrador siguen exigiéndolo. Store: `ms-windows-store://search/?query=<nombre del catálogo codificado>`; papelera: `shell:RecycleBinFolder`; ambos con `ShellExecuteExW`.

### Volver a descargar

`attrib +P -U "<ruta>"` ("Mantener siempre en este dispositivo"), validando que la ruta esté en una carpeta sincronizada detectada. Se registra en `cloud.log` como `descargado`.

### Interfaz

Filtro por módulo, entradas agrupadas por día (hoy, ayer, fecha), icono del módulo, texto en español, hora, error si lo hubo y botón de acción. Tras deshacer se recarga la lista.

## Risks / Trade-offs

- [Logs muy grandes] → Se muestran las 500 entradas más recientes.
- [Cambios hechos fuera de la app] → La validez se comprueba con el estado real, no con el log.
