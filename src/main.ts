import { invoke } from "@tauri-apps/api/core";
import { app, initApp } from "./app";
import { modules } from "./modules";
import type { Module } from "./modules/module";
import { registerNav } from "./nav";
import { escapeHtml } from "./ui";

const STORAGE_KEY = "activeModule";

const nav = document.querySelector<HTMLElement>("#nav")!;
const container = document.querySelector<HTMLElement>("#modules")!;
const panels = new Map<string, HTMLElement>();
const buttons = new Map<string, HTMLButtonElement>();

function savedModuleId(): string | null {
  try {
    return localStorage.getItem(STORAGE_KEY);
  } catch {
    return null;
  }
}

function saveModuleId(id: string) {
  try {
    localStorage.setItem(STORAGE_KEY, id);
  } catch {
    // Solo es una preferencia: si no se puede guardar, se abre el primero.
  }
}

/** Crea el panel de un módulo la primera vez que se abre; después solo se muestra. */
function panelFor(module: Module): HTMLElement {
  let panel = panels.get(module.id);
  if (!panel) {
    panel = document.createElement("section");
    panel.className = "panel";
    panel.setAttribute("aria-labelledby", `title-${module.id}`);
    panel.innerHTML = `
      <header class="module-header">
        <h2 id="title-${module.id}">${escapeHtml(module.label)}</h2>
        <p>${escapeHtml(module.description)}</p>
      </header>`;
    container.append(panel);
    module.render(panel);
    panels.set(module.id, panel);
  }
  return panel;
}

function activate(module: Module) {
  const panel = panelFor(module);
  for (const p of panels.values()) p.hidden = p !== panel;
  for (const [id, button] of buttons) {
    const active = id === module.id;
    button.classList.toggle("active", active);
    button.tabIndex = active ? 0 : -1;
    if (active) button.setAttribute("aria-current", "page");
    else button.removeAttribute("aria-current");
  }
  saveModuleId(module.id);
}

function buildNav() {
  for (const module of modules) {
    const button = document.createElement("button");
    button.className = "nav-item";
    button.title = module.label;
    button.innerHTML = `${module.icon}<span class="nav-label">${escapeHtml(module.label)}</span>`;
    button.addEventListener("click", () => activate(module));
    nav.append(button);
    buttons.set(module.id, button);
  }

  // Flechas, Inicio y Fin mueven el foco; Intro y Espacio activan (comportamiento nativo del botón).
  nav.addEventListener("keydown", (e) => {
    const items = [...buttons.values()];
    const current = items.indexOf(document.activeElement as HTMLButtonElement);
    if (current < 0) return;
    const next = {
      ArrowDown: (current + 1) % items.length,
      ArrowUp: (current - 1 + items.length) % items.length,
      Home: 0,
      End: items.length - 1,
    }[e.key];
    if (next === undefined) return;
    e.preventDefault();
    items.forEach((b, i) => (b.tabIndex = i === next ? 0 : -1));
    items[next].focus();
  });
}

function setBadge(moduleId: string, count: number) {
  const button = buttons.get(moduleId);
  if (!button) return;
  let badge = button.querySelector<HTMLElement>(".nav-badge");
  if (count <= 0) {
    badge?.remove();
    return;
  }
  if (!badge) {
    badge = document.createElement("span");
    badge.className = "nav-badge";
    button.append(badge);
  }
  badge.textContent = count > 99 ? "99+" : String(count);
  badge.title = `${count} pendientes`;
}

function setupAdminBanner() {
  document.querySelector<HTMLElement>("#admin-banner")!.hidden = app.isAdmin;
  document.querySelector<HTMLButtonElement>("#btn-elevate")!.addEventListener("click", async () => {
    try {
      await invoke("relaunch_as_admin");
    } catch (e) {
      document.querySelector("#admin-banner-text")!.textContent = String(e);
    }
  });
}

async function init() {
  await initApp();
  setupAdminBanner();
  buildNav();
  registerNav({
    navigate: (id) => {
      const module = modules.find((m) => m.id === id);
      if (module) activate(module);
    },
    setBadge,
  });
  const saved = modules.find((m) => m.id === savedModuleId());
  activate(saved ?? modules[0]);
  for (const module of modules) module.init?.();
}

init();
