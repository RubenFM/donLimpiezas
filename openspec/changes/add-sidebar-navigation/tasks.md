# Tasks

## 1. Base compartida

- [x] 1.1 Extraer `formatBytes`, `escapeHtml`, `groupBy`, `showStatus`, `setBusy` y `confirmDialog` a `src/ui.ts`, y el estado `isAdmin` con su inicialización a `src/app.ts`; verificar que `npx tsc --noEmit` pasa
- [x] 1.2 Definir la interfaz `Module` y el registro ordenado en `src/modules/index.ts`; verificar que `npx tsc --noEmit` pasa

## 2. Migrar los módulos existentes

- [ ] 2.1 Mover la limpieza a `src/modules/custom-clean.ts` (HTML como plantilla, selectores dentro de `root`); verificar con `npm run tauri dev` que analizar y limpiar funcionan igual que antes
- [ ] 2.2 Mover las apps preinstaladas a `src/modules/bloatware-remover.ts`; verificar con `npm run tauri dev` que buscar apps lista las mismas que antes

## 3. Barra lateral

- [x] 3.1 Reescribir `index.html` con el layout (barra lateral, aviso de permisos, área de contenido, diálogo) y `src/main.ts` como arranque: construir la barra a partir del registro, renderizado perezoso y cambio de módulo; verificar que `npx tsc --noEmit` y `npx vite build` pasan
- [ ] 3.2 Guardar y restaurar el módulo activo en `localStorage` con `try/catch` y caída al primer módulo; verificar cerrando y reabriendo la app con Bloatware Remover activo
- [ ] 3.3 Navegación con flechas, `aria-current` y foco visible; verificar recorriendo la barra solo con teclado
- [x] 3.4 Dibujar los iconos SVG de Custom Clean y Bloatware Remover; verificar que se ven en modo claro y oscuro
- [x] 3.5 Estilos de la barra lateral y versión contraída por debajo de 860 px; verificar redimensionando la ventana a 800 px

## 4. Integración y documentación

- [ ] 4.1 Comprobar los escenarios de `specs/app-navigation/spec.md` con `npm run tauri dev`, incluido que el estado de Custom Clean se conserva al cambiar de módulo y volver
- [x] 4.2 Actualizar la sección "Estructura" del `README.md` con `src/modules/` y cómo añadir un módulo; verificar que los pasos descritos coinciden con el código

## Workflow follow-up

- Archivar el cambio con `/opsx:archive` tras revisarlo, para crear `openspec/specs/app-navigation`.
