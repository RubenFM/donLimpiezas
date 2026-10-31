import { invoke } from "@tauri-apps/api/core";
import { escapeHtml, setBusy, showStatus } from "../ui";
import { icon, type Module } from "./module";

type Entry = {
  id: string;
  time: string;
  module: string;
  text: string;
  detail: string | null;
  ok: boolean;
  error: string | null;
  action: string | null;
};

/** Módulos que registran cambios, con el nombre de la barra lateral. */
const MODULES: Record<string, string> = {
  "bloatware-remover": "Bloatware Remover",
  uninstaller: "Uninstaller",
  "duplicate-finder": "Duplicate Finder",
  "cloud-drive-cleaner": "Cloud Drive Cleaner",
  "performance-optimizer": "Performance Optimizer",
  "startup-manager": "Startup Manager",
  "software-updater": "Software Updater",
  "driver-updater": "Driver Updater",
};

const template = `
  <div class="toolbar">
    <select data-el="filter" class="input" aria-label="Módulo">
      <option value="">Todos los módulos</option>
      ${Object.entries(MODULES)
        .map(([id, label]) => `<option value="${id}">${label}</option>`)
        .join("")}
    </select>
    <button data-el="refresh" class="btn">Actualizar</button>
    <span data-el="summary" class="summary"></span>
  </div>
  <div data-el="status" class="status" hidden></div>
  <div data-el="list" class="list">
    <p class="empty">Cargando historial…</p>
  </div>`;

function dayLabel(date: Date): string {
  const today = new Date();
  const yesterday = new Date(today);
  yesterday.setDate(today.getDate() - 1);
  const same = (a: Date, b: Date) => a.toDateString() === b.toDateString();
  if (same(date, today)) return "Hoy";
  if (same(date, yesterday)) return "Ayer";
  return date.toLocaleDateString("es-ES", { weekday: "long", day: "numeric", month: "long", year: "numeric" });
}

function render(root: HTMLElement) {
  root.insertAdjacentHTML("beforeend", template);
  const el = <T extends HTMLElement>(name: string) => root.querySelector(`[data-el="${name}"]`) as T;
  const list = el("list");
  const status = el("status");
  const filter = el<HTMLSelectElement>("filter");
  const btnRefresh = el<HTMLButtonElement>("refresh");

  let entries: Entry[] = [];

  function renderEntries() {
    const visible = entries.filter((e) => !filter.value || e.module === filter.value);
    el("summary").textContent = `${visible.length} ${visible.length === 1 ? "acción" : "acciones"}`;
    if (visible.length === 0) {
      list.innerHTML = '<p class="empty">Todavía no hay cambios registrados.</p>';
      return;
    }
    let lastDay = "";
    list.innerHTML = visible
      .map((e) => {
        const date = new Date(e.time);
        const day = dayLabel(date);
        const header = day !== lastDay ? `<h3 class="group">${escapeHtml(day)}</h3>` : "";
        lastDay = day;
        const time = date.toLocaleTimeString("es-ES", { hour: "2-digit", minute: "2-digit" });
        return `${header}
          <div class="row">
            <span class="card-mark history-mark ${e.ok ? "ok" : "warn"}" aria-hidden="true">${e.ok ? "✓" : "!"}</span>
            <div class="row-main">
              <div class="row-title">${escapeHtml(e.text)}
                <span class="pill">${escapeHtml(MODULES[e.module] ?? e.module)}</span></div>
              ${e.detail ? `<div class="row-desc" title="${escapeHtml(e.detail)}">${escapeHtml(e.detail)}</div>` : ""}
              ${e.error ? `<div class="row-warning">${escapeHtml(e.error)}</div>` : ""}
            </div>
            ${e.action ? `<button class="btn small" data-undo="${escapeHtml(e.id)}">${escapeHtml(e.action)}</button>` : ""}
            <div class="row-size"><span>${time}</span></div>
          </div>`;
      })
      .join("");
  }

  async function load() {
    setBusy(btnRefresh, true, "Cargando…");
    try {
      entries = await invoke<Entry[]>("list_history");
      renderEntries();
    } catch (e) {
      showStatus(status, escapeHtml(String(e)), "error");
    } finally {
      setBusy(btnRefresh, false);
    }
  }

  list.addEventListener("click", async (event) => {
    const button = (event.target as HTMLElement).closest<HTMLButtonElement>("[data-undo]");
    if (!button) return;
    setBusy(button, true, "…");
    try {
      showStatus(status, escapeHtml(await invoke<string>("undo_history", { id: button.dataset.undo })), "ok");
    } catch (e) {
      showStatus(status, escapeHtml(String(e)), "error");
    }
    await load();
  });
  filter.addEventListener("change", renderEntries);
  btnRefresh.addEventListener("click", load);
  load();
}

export const history: Module = {
  id: "history",
  label: "History",
  description: "Todo lo que la app ha cambiado en tu equipo y cómo deshacerlo.",
  // Reloj con flecha hacia atrás.
  icon: icon(
    `<path d="M4 12a8 8 0 1 0 2.3-5.7"/>
     <path d="M4 4v4.5h4.5"/>
     <path d="M12 8v4.5l3 1.8"/>`,
  ),
  render,
};
