# Proposal

## Why

Cuando el disco se llena, lo primero es saber qué ocupa espacio. Ningún módulo actual lo responde: limpian categorías concretas, pero no muestran dónde están los gigas.

## What Changes

- Nuevo módulo **Disk Analyzer** en la barra lateral, antes de Duplicate Finder.
- Analiza un disco fijo o una carpeta elegida, en segundo plano con progreso y cancelación.
- Muestra el contenido de la carpeta actual ordenado por tamaño con barras proporcionales; se navega entrando en carpetas y con una ruta de migas.
- Muestra los archivos más grandes de cada carpeta y agrupa el resto.
- Botón para abrir la carpeta en el Explorador. El módulo no borra nada.
- Fuera de alcance: mapa de rectángulos (treemap), comparar análisis y borrar desde aquí.

## Capabilities

### New Capabilities
- `disk-analysis`: análisis del espacio ocupado por carpetas y navegación por el resultado.

### Modified Capabilities

## Impact

- Nuevo `src-tauri/src/diskusage.rs`, comandos `analyze_disk`, `cancel_disk_analysis`, `disk_children` y `open_in_explorer`, y evento de progreso.
- Nuevo `src/modules/disk-analyzer.ts`.
