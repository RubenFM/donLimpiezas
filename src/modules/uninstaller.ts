import { invoke } from "@tauri-apps/api/core";
import { confirmDialog, escapeHtml, formatBytes, setBusy, showStatus } from "../ui";
import { icon, type Module } from "./module";

type Program = {
  id: string;
  name: string;
  publisher: string | null;
  version: string | null;
  size: number | null;
  installDate: string | null;
};

type UninstallResult = { removed: boolean };

type SortKey = "name" | "size" | "date";

const template = `
  <div class="toolbar">
    <input data-el="search" class="input" type="search" placeholder="Buscar por nombre o editor" />
    <select data-el="sort" class="input" aria-label="Ordenar">
      <option value="name">Nombre</option>
      <option value="size">Tamaño</option>
      <option value="date">Fecha de instalación</option>
    </select>
    <button data-el="refresh" class="btn">Actualizar</button>
    <button data-el="uninstall" class="btn danger" disabled>Desinstalar seleccionados</button>
    <span data-el="summary" class="summary"></span>
  </div>
  <div data-el="list" class="list">
    <p class="empty">Cargando programas instalados…</p>
  </div>
  <div data-el="status" class="status" hidden></div>`;

const compare: Record<SortKey, (a: Program, b: Program) => number> = {
  name: (a, b) => a.name.localeCompare(b.name, "es", { sensitivity: "base" }),
  // Sin tamaño o sin fecha, al final.
  size: (a, b) => (b.size ?? -1) - (a.size ?? -1),
  date: (a, b) => (b.installDate ?? "").localeCompare(a.installDate ?? ""),
};

function render(root: HTMLElement) {
  root.insertAdjacentHTML("beforeend", template);
  const el = <T extends HTMLElement>(name: string) => root.querySelector(`[data-el="${name}"]`) as T;
  const list = el("list");
  const status = el("status");
  const search = el<HTMLInputElement>("search");
  const sort = el<HTMLSelectElement>("sort");
  const btnRefresh = el<HTMLButtonElement>("refresh");
  const btnUninstall = el<HTMLButtonElement>("uninstall");

  let programs: Program[] = [];
  // La selección se conserva al buscar u ordenar.
  const selected = new Set<string>();

  function visible(): Program[] {
    const q = search.value.trim().toLowerCase();
    return programs
      .filter((p) => !q || p.name.toLowerCase().includes(q) || p.publisher?.toLowerCase().includes(q))
      .sort(compare[sort.value as SortKey]);
  }

  function updateSummary() {
    const total = programs.reduce((s, p) => s + (p.size ?? 0), 0);
    el("summary").textContent = programs.length
      ? `${programs.length} programas · ${formatBytes(total)}`
      : "";
    const n = selected.size;
    btnUninstall.disabled = n === 0;
    btnUninstall.textContent = n ? `Desinstalar ${n} seleccionado${n === 1 ? "" : "s"}` : "Desinstalar seleccionados";
  }

  function renderPrograms() {
    const items = visible();
    if (items.length === 0) {
      list.innerHTML = `<p class="empty">${
        programs.length ? "Ningún programa coincide con la búsqueda." : "No se han encontrado programas."
      }</p>`;
      updateSummary();
      return;
    }
    list.innerHTML = items
      .map((p) => {
        const details = [p.publisher, p.version && `v${p.version}`, p.installDate && `instalado el ${p.installDate}`]
          .filter(Boolean)
          .map((d) => escapeHtml(d!));
        return `
          <label class="row">
            <input type="checkbox" value="${escapeHtml(p.id)}" ${selected.has(p.id) ? "checked" : ""} />
            <div class="row-main">
              <div class="row-title">${escapeHtml(p.name)}</div>
              <div class="row-desc">${details.join(" · ")}</div>
            </div>
            <div class="row-size"><strong>${p.size ? formatBytes(p.size) : "—"}</strong></div>
          </label>`;
      })
      .join("");
    updateSummary();
  }

  async function refresh() {
    setBusy(btnRefresh, true, "Actualizando…");
    try {
      programs = await invoke<Program[]>("list_programs");
      const ids = new Set(programs.map((p) => p.id));
      for (const id of selected) if (!ids.has(id)) selected.delete(id);
      renderPrograms();
    } catch (e) {
      showStatus(status, `Error al listar los programas: ${escapeHtml(String(e))}`, "error");
    } finally {
      setBusy(btnRefresh, false);
    }
  }

  async function uninstallSelected() {
    const queue = programs.filter((p) => selected.has(p.id));
    const ok = await confirmDialog(
      "¿Desinstalar estos programas?",
      `Se abrirá el desinstalador de cada uno, de uno en uno:<br>• ${queue
        .map((p) => escapeHtml(p.name))
        .join("<br>• ")}<br><br>Sigue sus instrucciones; puedes cancelar cualquiera desde su propio asistente.`,
    );
    if (!ok) return;

    const controls = [search, sort, btnRefresh];
    controls.forEach((c) => (c.disabled = true));
    setBusy(btnUninstall, true, "Desinstalando…");
    const lines: string[] = [];
    let removed = 0;
    for (const [i, p] of queue.entries()) {
      showStatus(
        status,
        `Desinstalando <strong>${escapeHtml(p.name)}</strong> (${i + 1} de ${queue.length})… ` +
          "Sigue las instrucciones del desinstalador.",
      );
      try {
        const result = await invoke<UninstallResult>("uninstall_program", { id: p.id });
        if (result.removed) {
          removed++;
          selected.delete(p.id);
          lines.push(`✓ ${escapeHtml(p.name)}: desinstalado.`);
        } else {
          lines.push(`• ${escapeHtml(p.name)}: sigue instalado (se canceló o el desinstalador sigue abierto).`);
        }
      } catch (e) {
        lines.push(`✗ ${escapeHtml(p.name)}: ${escapeHtml(String(e))}`);
      }
    }
    setBusy(btnUninstall, false);
    controls.forEach((c) => (c.disabled = false));
    showStatus(
      status,
      [`${removed} de ${queue.length} programas desinstalados.`, ...lines].join("<br>"),
      removed === queue.length ? "ok" : "error",
    );
    await refresh();
  }

  list.addEventListener("change", (event) => {
    const input = event.target as HTMLInputElement;
    if (input.checked) selected.add(input.value);
    else selected.delete(input.value);
    updateSummary();
  });
  search.addEventListener("input", renderPrograms);
  sort.addEventListener("change", renderPrograms);
  btnRefresh.addEventListener("click", refresh);
  btnUninstall.addEventListener("click", uninstallSelected);
  refresh();
}

export const uninstaller: Module = {
  id: "uninstaller",
  label: "Uninstaller",
  description: "Programas instalados, con su tamaño y fecha.",
  // Papelera.
  icon: icon(
    `<path d="M4 7h16"/>
     <path d="M9.5 7V4.5h5V7"/>
     <path d="M6.5 7l1 13h9l1-13"/>
     <path d="M10.5 11v5.5M13.5 11v5.5"/>`,
  ),
  render,
};
