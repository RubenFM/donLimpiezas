import { invoke } from "@tauri-apps/api/core";
import { confirmDialog, escapeHtml, setBusy, showStatus } from "../ui";
import { getUpdates, type Update } from "../updates";
import { icon, type Module } from "./module";

type UpdateResult = { ok: boolean; message: string | null };

const template = `
  <div class="toolbar">
    <button data-el="refresh" class="btn">Buscar actualizaciones</button>
    <button data-el="all" class="btn" disabled>Seleccionar todas</button>
    <button data-el="update" class="btn primary" disabled>Actualizar seleccionadas</button>
    <span data-el="summary" class="summary"></span>
  </div>
  <div data-el="list" class="list">
    <p class="empty">Buscando actualizaciones con winget…</p>
  </div>
  <div data-el="status" class="status" hidden></div>`;

function render(root: HTMLElement) {
  root.insertAdjacentHTML("beforeend", template);
  const el = <T extends HTMLElement>(name: string) => root.querySelector(`[data-el="${name}"]`) as T;
  const list = el("list");
  const status = el("status");
  const btnRefresh = el<HTMLButtonElement>("refresh");
  const btnAll = el<HTMLButtonElement>("all");
  const btnUpdate = el<HTMLButtonElement>("update");

  let updates: Update[] = [];
  const selected = new Set<string>();
  let busy = false;

  function updateSummary() {
    el("summary").textContent = updates.length ? `${updates.length} actualizaciones disponibles` : "";
    const n = selected.size;
    btnUpdate.disabled = busy || n === 0;
    btnUpdate.textContent = n ? `Actualizar ${n} seleccionada${n === 1 ? "" : "s"}` : "Actualizar seleccionadas";
    btnAll.disabled = busy || updates.length === 0;
    btnAll.textContent = n === updates.length && n > 0 ? "Quitar selección" : "Seleccionar todas";
  }

  function renderUpdates() {
    if (updates.length === 0) {
      list.innerHTML = '<p class="empty">Todos tus programas están al día. 🎉</p>';
      updateSummary();
      return;
    }
    list.innerHTML = updates
      .map(
        (u) => `
          <label class="row">
            <input type="checkbox" value="${escapeHtml(u.id)}" ${selected.has(u.id) ? "checked" : ""} ${busy ? "disabled" : ""} />
            <div class="row-main">
              <div class="row-title">${escapeHtml(u.name)}</div>
              <div class="row-desc">${escapeHtml(u.id)}</div>
            </div>
            <div class="row-size">
              <strong>${escapeHtml(u.available)}</strong>
              <span>instalada ${u.version === "Unknown" ? "desconocida" : escapeHtml(u.version)}</span>
            </div>
          </label>`,
      )
      .join("");
    updateSummary();
  }

  async function load(refresh: boolean) {
    setBusy(btnRefresh, true, "Buscando…");
    try {
      updates = await getUpdates(refresh);
      const ids = new Set(updates.map((u) => u.id));
      for (const id of selected) if (!ids.has(id)) selected.delete(id);
      renderUpdates();
    } catch (e) {
      list.innerHTML = "";
      showStatus(status, escapeHtml(String(e)), "error");
    } finally {
      setBusy(btnRefresh, false);
    }
  }

  async function updateSelected() {
    const queue = updates.filter((u) => selected.has(u.id));
    const ok = await confirmDialog(
      "¿Actualizar estos programas?",
      `Se descargará e instalará la versión nueva de cada uno, de uno en uno:<br>• ${queue
        .map((u) => `${escapeHtml(u.name)} (${escapeHtml(u.available)})`)
        .join("<br>• ")}<br><br>Algunos instaladores mostrarán su ventana o pedirán permisos.`,
    );
    if (!ok) return;

    busy = true;
    btnRefresh.disabled = true;
    renderUpdates();
    const lines: string[] = [];
    let done = 0;
    for (const [i, u] of queue.entries()) {
      showStatus(status, `Actualizando <strong>${escapeHtml(u.name)}</strong> (${i + 1} de ${queue.length})…`);
      try {
        const result = await invoke<UpdateResult>("update_program", { id: u.id });
        if (result.ok) {
          done++;
          selected.delete(u.id);
          lines.push(`✓ ${escapeHtml(u.name)}: actualizado a ${escapeHtml(u.available)}.`);
        } else {
          lines.push(`✗ ${escapeHtml(u.name)}: ${escapeHtml(result.message ?? "winget no pudo actualizarlo")}`);
        }
      } catch (e) {
        lines.push(`✗ ${escapeHtml(u.name)}: ${escapeHtml(String(e))}`);
      }
    }
    busy = false;
    showStatus(
      status,
      [`${done} de ${queue.length} programas actualizados.`, ...lines].join("<br>"),
      done === queue.length ? "ok" : "error",
    );
    await load(true);
  }

  list.addEventListener("change", (event) => {
    const input = event.target as HTMLInputElement;
    if (input.checked) selected.add(input.value);
    else selected.delete(input.value);
    updateSummary();
  });
  btnAll.addEventListener("click", () => {
    const all = selected.size === updates.length;
    selected.clear();
    if (!all) for (const u of updates) selected.add(u.id);
    renderUpdates();
  });
  btnRefresh.addEventListener("click", () => load(true));
  btnUpdate.addEventListener("click", updateSelected);
  load(false);
}

export const softwareUpdater: Module = {
  id: "software-updater",
  label: "Software Updater",
  description: "Actualiza tus programas con winget y sus instaladores oficiales.",
  // Flecha circular.
  icon: icon(
    `<path d="M20 12a8 8 0 1 1-2.4-5.7"/>
     <path d="M20 4v4.5h-4.5"/>
     <path d="M12 8v4.5l3 1.8"/>`,
  ),
  render,
  // Comprueba al arrancar para mostrar el contador sin abrir el módulo.
  init: () => {
    getUpdates().catch(() => {});
  },
};
