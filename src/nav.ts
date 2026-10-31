// Puente entre los módulos y la barra lateral, que implementa main.ts.
// Evita que los módulos importen main.ts (importación circular).

type NavImpl = {
  navigate(moduleId: string): void;
  setBadge(moduleId: string, count: number): void;
};

let impl: NavImpl | null = null;

export function registerNav(nav: NavImpl) {
  impl = nav;
}

/** Abre otro módulo, igual que al pulsarlo en la barra lateral. */
export function navigate(moduleId: string) {
  impl?.navigate(moduleId);
}

/** Muestra un contador junto al módulo; con 0 se oculta. */
export function setBadge(moduleId: string, count: number) {
  impl?.setBadge(moduleId, count);
}
