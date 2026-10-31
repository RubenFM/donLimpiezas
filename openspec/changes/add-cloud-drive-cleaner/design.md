# Design

## Context

Ver `proposal.md`. OneDrive y Dropbox usan la Cloud Files API de Windows: los archivos son puntos de reanálisis con atributos que indican su estado. En el equipo de desarrollo, OneDrive personal está en `C:\Users\ruben\OneDrive` con los archivos descargados (`Archive, ReparsePoint`).

## Goals / Non-Goals

**Goals:**
- Hacer lo mismo que "Liberar espacio" del Explorador, de forma masiva y guiada.

**Non-Goals:**
- Hablar con los servicios en la nube.

## Decisions

### Carpetas

- OneDrive: `UserFolder` de cada subclave de `HKCU\Software\Microsoft\OneDrive\Accounts` (la subclave da el nombre: `Personal`, `Business1`…). Se omiten las vacías.
- Dropbox: `path` de `personal` y `business` en `%LOCALAPPDATA%\Dropbox\info.json` (o `%APPDATA%`).

### Estado de cada archivo (solo atributos, sin abrirlo)

| Atributo | Valor | Significado |
|---|---|---|
| `RECALL_ON_DATA_ACCESS` / `OFFLINE` | `0x400000` / `0x1000` | Solo en la nube: se ignora |
| `PINNED` | `0x80000` | "Mantener siempre en este dispositivo": se muestra como "Fijado" |
| `UNPINNED` | `0x100000` | Ya pedido "liberar espacio": se ignora |

No se siguen enlaces ni uniones (como en Duplicate Finder).

### Liberar con `attrib`

`attrib -P +U "<ruta>"` es la forma documentada y soportada de pedir al proveedor de sincronización que deshidrate un archivo. Se ejecuta por archivo, sin ventana. Alternativa descartada: `CfDehydratePlaceholder`, que exige abrir el archivo con permisos especiales y gestionar errores del proveedor. La deshidratación la completa OneDrive en segundo plano; se considera liberado si `attrib` termina bien.

### Validación

El último análisis se guarda en un estado de Tauri; solo se aceptan rutas de él y dentro de una carpeta detectada.

### Interfaz

Selector de carpeta (todas o una), tamaño mínimo (1, 10, 100 MB) y antigüedad (cualquiera, 30, 90, 365 días). Lista con nombre, carpeta, tamaño y fecha; "Seleccionar todo" y "Liberar espacio". Resumen: espacio local total de las carpetas y espacio seleccionado.

## Risks / Trade-offs

- [El usuario libera un archivo que necesita sin conexión] → Se avisa en la confirmación; se descarga al abrirlo con conexión.
- [Un archivo pendiente de subir no se puede liberar] → `attrib` falla y se informa; el archivo sigue intacto.
- [Fijados: liberarlos cambia la preferencia del usuario] → Aparecen desmarcados con etiqueta.
