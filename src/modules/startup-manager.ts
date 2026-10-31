import { invoke } from "@tauri-apps/api/core";
import { app } from "../app";
import { escapeHtml, groupBy, setBusy, showStatus } from "../ui";
import { icon, type Module } from "./module";

type StartupEntry = {
  source: string;
  sourceLabel: string;
  name: string;
  displayName: string;
  command: string;
  target: string | null;
  publisher: string | null;
  enabled: boolean;
  admin: boolean;
};

/** Explicación de los orígenes que no se comportan como la pestaña Inicio. */
const NOTES: Record<string, string> = {
  scheduled_task: "Tareas del Programador de tareas que se ejecutan al iniciar sesión o al arrancar el equipo.",
  service:
    "Desactivar un servicio lo pasa a inicio manual: Windows lo arranca si un programa lo necesita. " +
    "Surte efecto al reiniciar.",
};

const template = `
  <div class="toolbar">
    <button data-el="refresh" class="btn">Actualizar</button>
    <span data-el="summary" class="summary"></span>
  </div>
  <div data-el="list" class="list">
    <p class="empty">Cargando programas de inicio…</p>
  </div>
  <div data-el="status" class="status" hidden></div>`;

function render(root: HTMLElement) {
  root.insertAdjacentHTML("beforeend", template);
  const el = <T extends HTMLElement>(name: string) => root.querySelector(`[data-el="${name}"]`) as T;
  const list = el("list");
  const status = el("status");
  const btnRefresh = el<HTMLButtonElement>("refresh");

  let entries: StartupEntry[] = [];

  function updateSummary() {
    const enabled = entries.filter((e) => e.enabled).length;
    el("summary").textContent = entries.length
      ? `${enabled} de ${entries.length} se inician con Windows`
      : "";
  }

  function renderEntries() {
    if (entries.length === 0) {
      list.innerHTML = '<p class="empty">No hay programas configurados para iniciarse con Windows.</p>';
      updateSummary();
      return;
    }
    const grouped = groupBy(entries.map((e, index) => ({ ...e, index, group: e.sourceLabel })));
    list.innerHTML = [...grouped]
      .map(([group, items]) => {
        const rows = items
          .map((e) => {
            const locked = e.admin && !app.isAdmin;
            const details = [e.publisher, e.target ?? e.command].filter(Boolean).map((d) => escapeHtml(d!));
            return `
              <div class="row ${locked ? "locked" : ""}">
                <div class="row-main">
                  <div class="row-title">${escapeHtml(e.displayName)}
                    ${e.admin ? '<span class="pill">Admin</span>' : ""}</div>
                  <div class="row-desc" title="${escapeHtml(e.command)}">${details.join(" · ")}</div>
                </div>
                <label class="switch" title="${e.enabled ? "Desactivar" : "Activar"}">
                  <input type="checkbox" role="switch" data-index="${e.index}"
                    aria-label="Iniciar ${escapeHtml(e.displayName)} con Windows"
                    ${e.enabled ? "checked" : ""} ${locked ? "disabled" : ""} />
                  <span class="slider"></span>
                </label>
              </div>`;
          })
          .join("");
        const note = NOTES[items[0].source];
        return `<h3 class="group">${escapeHtml(group)}</h3>
          ${note ? `<p class="row-note group-note">${note}</p>` : ""}${rows}`;
      })
      .join("");
    updateSummary();
  }

  async function refresh() {
    setBusy(btnRefresh, true, "Actualizando…");
    try {
      entries = await invoke<StartupEntry[]>("list_startup");
      renderEntries();
    } catch (e) {
      showStatus(status, `Error al listar los programas de inicio: ${escapeHtml(String(e))}`, "error");
    } finally {
      setBusy(btnRefresh, false);
    }
  }

  // Sin confirmación: no es destructivo y se deshace con otro clic.
  list.addEventListener("change", async (event) => {
    const input = event.target as HTMLInputElement;
    const entry = entries[Number(input.dataset.index)];
    if (!entry) return;
    const enabled = input.checked;
    input.disabled = true;
    try {
      await invoke("set_startup_enabled", { source: entry.source, name: entry.name, enabled });
      entry.enabled = enabled;
      input.closest(".switch")!.setAttribute("title", enabled ? "Desactivar" : "Activar");
      status.hidden = true;
    } catch (e) {
      input.checked = !enabled;
      showStatus(status, escapeHtml(String(e)), "error");
    } finally {
      input.disabled = false;
      updateSummary();
    }
  });

  btnRefresh.addEventListener("click", refresh);
  refresh();
}

export const startupManager: Module = {
  id: "startup-manager",
  label: "Startup Manager",
  description: "Programas, tareas programadas y servicios que arrancan con Windows.",
  // Botón de encendido.
  icon: icon(
    `<path d="M12 3v8"/>
     <path d="M6.3 6.8a8 8 0 1 0 11.4 0"/>`,
  ),
  render,
};
