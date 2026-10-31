# Design

## Context

Ver `proposal.md`. Hoy todo el frontend está en `src/main.ts` (~330 líneas) con dos paneles fijos en `index.html` y pestañas en la cabecera. Cada módulo nuevo añadiría su HTML a `index.html` y su lógica al mismo archivo.

## Goals / Non-Goals

**Goals:**
- Que añadir un módulo sea crear un archivo en `src/modules/` y registrarlo en una lista.
- Mantener TypeScript sin framework.

**Non-Goals:**
- Rutas por URL o historial (atrás/adelante).
- Rediseño visual de los módulos existentes más allá de encajarlos en el nuevo layout.

## Decisions

### Contrato de módulo

```ts
interface Module {
  id: string;            // "custom-clean", estable: se guarda en localStorage
  label: string;         // "Custom Clean"
  description: string;   // subtítulo en español, en la cabecera del módulo
  icon: string;          // SVG en línea
  render(root: HTMLElement): void;
}
```

`src/modules/index.ts` exporta el array ordenado de módulos implementados. `src/main.ts` solo construye la barra lateral, comprueba los permisos y activa el módulo inicial.

### Renderizado perezoso y paneles persistentes

Cada módulo se renderiza la **primera vez** que se abre, en su propio `<section>`, y después solo se oculta o muestra. Así se conserva su estado (análisis, casillas) sin un sistema de estado explícito. Alternativa descartada: volver a renderizar al cambiar, que obligaría a guardar y restaurar el estado de cada módulo.

El HTML de cada módulo pasa de `index.html` a su archivo, como plantilla dentro de `render`. `index.html` queda con el layout (barra lateral, aviso de permisos, área de contenido) y el diálogo de confirmación.

### Utilidades compartidas

`formatBytes`, `escapeHtml`, `groupBy`, `showStatus`, `setBusy` y `confirmDialog` pasan a `src/ui.ts`. El estado global mínimo (`isAdmin`) va en `src/app.ts`, que se inicializa antes de renderizar ningún módulo.

### Persistencia del módulo activo

`localStorage["activeModule"]` envuelto en `try/catch`: si falla la lectura o el id ya no existe, se abre el primer módulo. Es una preferencia de comodidad, no un dato crítico, así que no merece un archivo de configuración en disco.

### Accesibilidad y teclado

La barra lateral es un `<nav>` con un `<button>` por módulo y `aria-current="page"` en el activo. Flechas arriba y abajo mueven el foco entre botones (roving `tabindex`); Intro y Espacio activan el botón, que es su comportamiento nativo.

### Layout

Barra lateral de 220 px con icono y nombre. Por debajo de 860 px de ancho se contrae a solo iconos, con el nombre como `title`. La ventana mantiene su `minWidth` de 780 px.

### Iconos

SVG propios de 20×20 con trazo (`stroke="currentColor"`) para heredar el color del tema. Solo se dibujan los de los módulos implementados en cada cambio.

## Risks / Trade-offs

- [Mover HTML a plantillas en TS rompe sin aviso algún selector] → Cada módulo busca sus elementos dentro de su `root` en vez de en `document`, y se prueba cada módulo a mano tras la migración.
- [Muchos módulos renderizados a la vez consumen memoria] → Despreciable con diez módulos de listas sencillas.
- [`localStorage` distinto entre `tauri dev` y la build final (orígenes distintos)] → Aceptable: solo afecta al módulo que se abre al iniciar.
