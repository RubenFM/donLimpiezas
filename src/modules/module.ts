export interface Module {
  /** Identificador estable: se guarda en localStorage. */
  id: string;
  /** Nombre en la barra lateral (en inglés). */
  label: string;
  /** Subtítulo en español bajo el nombre del módulo. */
  description: string;
  /** SVG en línea de 24×24 con trazo `currentColor`. */
  icon: string;
  /** Se llama una sola vez, la primera vez que se abre el módulo. */
  render(root: HTMLElement): void;
}

/** Envuelve los trazos de un icono en el SVG común. */
export function icon(paths: string): string {
  return `<svg viewBox="0 0 24 24" width="20" height="20" fill="none" stroke="currentColor"
    stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">${paths}</svg>`;
}
