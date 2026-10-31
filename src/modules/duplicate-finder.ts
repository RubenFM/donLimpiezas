import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { open } from "@tauri-apps/plugin-dialog";
import { confirmDialog, escapeHtml, formatBytes, showStatus } from "../ui";
import { icon, type Module } from "./module";

type Progress = { phase: "scanning" | "comparing" | "verifying"; found: number; processed: number; total: number };
type DuplicateFile = { path: string; modified: number | null };
type Group = { size: number; files: DuplicateFile[] };
type ScanResult = { groups: Group[]; totalGroups: number; wasted: number };

const MIN_SIZES: [number, string][] = [
  [100 * 1024, "100 KB"],
  [1024 * 1024, "1 MB"],
  [10 * 1024 * 1024, "10 MB"],
  [100 * 1024 * 1024, "100 MB"],
];

const template = `
  <div class="folders">
    <div data-el="folders" class="chips"></div>
    <button data-el="add" class="btn small">Añadir carpeta</button>
  </div>
  <div class="toolbar">
    <label class="option">Tamaño mínimo
      <select data-el="min" class="input">
        ${MIN_SIZES.map(([v, l]) => `<option value="${v}" ${l === "1 MB" ? "selected" : ""}>${l}</option>`).join("")}
      </select>
    </label>
    <button data-el="search" class="btn">Buscar duplicados</button>
    <button data-el="cancel" class="btn" hidden>Cancelar</button>
    <button data-el="oldest" class="btn" disabled>Conservar el más antiguo</button>
    <button data-el="trash" class="btn danger" disabled>Enviar a la papelera</button>
    <span data-el="summary" class="summary"></span>
  </div>
  <div data-el="progress" class="progress" hidden><div class="progress-bar"></div><span></span></div>
  <div data-el="list" class="list">
    <p class="empty">Elige las carpetas y pulsa <strong>Buscar duplicados</strong>.</p>
  </div>
  <div data-el="status" class="status" hidden></div>`;

function splitPath(path: string): [string, string] {
  const i = path.lastIndexOf("\\");
  return [path.slice(0, i), path.slice(i + 1)];
}

function formatDate(secs: number | null): string {
  return secs ? new Date(secs * 1000).toLocaleDateString("es-ES") : "—";
}

function render(root: HTMLElement) {
  root.insertAdjacentHTML("beforeend", template);
  const el = <T extends HTMLElement>(name: string) => root.querySelector(`[data-el="${name}"]`) as T;
  const list = el("list");
  const status = el("status");
  const progress = el("progress");
  const btnSearch = el<HTMLButtonElement>("search");
  const btnCancel = el<HTMLButtonElement>("cancel");
  const btnOldest = el<HTMLButtonElement>("oldest");
  const btnTrash = el<HTMLButtonElement>("trash");
  const btnAdd = el<HTMLButtonElement>("add");
  const minSize = el<HTMLSelectElement>("min");

  let folders: string[] = [];
  let result: ScanResult | null = null;
  const selected = new Set<string>();
  let scanning = false;

  function renderFolders() {
    el("folders").innerHTML = folders.length
      ? folders
          .map(
            (f, i) =>
              `<span class="chip" title="${escapeHtml(f)}">${escapeHtml(f)}
                <button data-remove="${i}" aria-label="Quitar ${escapeHtml(f)}" ${scanning ? "disabled" : ""}>×</button></span>`,
          )
          .join("")
      : '<span class="row-note">Ninguna carpeta seleccionada.</span>';
    btnSearch.disabled = scanning || folders.length === 0;
  }

  /** Grupos en los que se han marcado todas las copias. */
  function fullySelectedGroups(): number {
    return result?.groups.filter((g) => g.files.every((f) => selected.has(f.path))).length ?? 0;
  }

  function updateSummary() {
    const freed = result?.groups.reduce((s, g) => s + g.size * g.files.filter((f) => selected.has(f.path)).length, 0) ?? 0;
    el("summary").textContent = result
      ? `${result.totalGroups} grupos · ${formatBytes(result.wasted)} recuperables` +
        (selected.size ? ` · ${selected.size} seleccionados (${formatBytes(freed)})` : "")
      : "";
    btnOldest.disabled = scanning || !result?.groups.length;
    btnTrash.disabled = scanning || selected.size === 0;
  }

  function renderGroups() {
    if (!result) return;
    if (result.groups.length === 0) {
      list.innerHTML = '<p class="empty">No se han encontrado archivos duplicados. 🎉</p>';
      updateSummary();
      return;
    }
    const more =
      result.totalGroups > result.groups.length
        ? `<p class="row-note list-note">Mostrando los ${result.groups.length} grupos con más espacio de ${result.totalGroups}.</p>`
        : "";
    list.innerHTML =
      result.groups
        .map((g, gi) => {
          const full = g.files.every((f) => selected.has(f.path));
          const rows = g.files
            .map((f) => {
              const [dir, name] = splitPath(f.path);
              return `
                <label class="row">
                  <input type="checkbox" data-group="${gi}" value="${escapeHtml(f.path)}" ${selected.has(f.path) ? "checked" : ""} />
                  <div class="row-main">
                    <div class="row-title">${escapeHtml(name)}</div>
                    <div class="row-desc" title="${escapeHtml(f.path)}">${escapeHtml(dir)}</div>
                  </div>
                  <div class="row-size"><span>${formatDate(f.modified)}</span></div>
                </label>`;
            })
            .join("");
          return `<h3 class="group">${formatBytes(g.size)} × ${g.files.length} copias</h3>
            ${full ? '<div class="row-warning group-warning">Has seleccionado todas las copias: conserva al menos una.</div>' : ""}
            ${rows}`;
        })
        .join("") + more;
    updateSummary();
  }

  function setScanning(on: boolean) {
    scanning = on;
    btnSearch.hidden = on;
    btnCancel.hidden = !on;
    btnAdd.disabled = on;
    minSize.disabled = on;
    progress.hidden = !on;
    renderFolders();
    updateSummary();
  }

  function showProgress(p: Progress) {
    const text = {
      scanning: `Buscando archivos… ${p.found.toLocaleString("es-ES")} encontrados`,
      comparing: `Comparando archivos del mismo tamaño… ${p.processed.toLocaleString("es-ES")} de ${p.total.toLocaleString("es-ES")}`,
      verifying: `Verificando el contenido completo… ${p.processed.toLocaleString("es-ES")} de ${p.total.toLocaleString("es-ES")}`,
    }[p.phase];
    progress.querySelector("span")!.textContent = text;
    const bar = progress.querySelector<HTMLElement>(".progress-bar")!;
    bar.classList.toggle("indeterminate", p.phase === "scanning");
    bar.style.width = p.phase === "scanning" || !p.total ? "" : `${(100 * p.processed) / p.total}%`;
  }

  async function search() {
    selected.clear();
    result = null;
    status.hidden = true;
    list.innerHTML = "";
    setScanning(true);
    showProgress({ phase: "scanning", found: 0, processed: 0, total: 0 });
    try {
      result = await invoke<ScanResult>("find_duplicates", { folders, minSize: Number(minSize.value) });
      renderGroups();
    } catch (e) {
      list.innerHTML = "";
      showStatus(status, escapeHtml(String(e)), "info");
    } finally {
      setScanning(false);
    }
  }

  function keepOldest() {
    if (!result) return;
    selected.clear();
    for (const g of result.groups) {
      const oldest = g.files.reduce((a, b) => ((b.modified ?? Infinity) < (a.modified ?? Infinity) ? b : a));
      for (const f of g.files) if (f !== oldest) selected.add(f.path);
    }
    renderGroups();
  }

  async function trash() {
    if (fullySelectedGroups() > 0) {
      showStatus(status, "En algún grupo has seleccionado todas las copias. Conserva al menos una de cada.", "error");
      return;
    }
    const paths = [...selected];
    const ok = await confirmDialog(
      "¿Enviar a la papelera?",
      `Se enviarán ${paths.length} archivos a la papelera de reciclaje. Podrás recuperarlos desde allí.`,
    );
    if (!ok) return;
    btnTrash.disabled = true;
    try {
      const { trashed } = await invoke<{ trashed: number }>("trash_duplicates", { paths });
      // Quita lo enviado a la papelera y los grupos que se quedan con una sola copia.
      if (result) {
        const gone = new Set(paths);
        result.groups = result.groups
          .map((g) => ({ ...g, files: g.files.filter((f) => !gone.has(f.path)) }))
          .filter((g) => g.files.length > 1);
        result.totalGroups = result.groups.length;
        result.wasted = result.groups.reduce((s, g) => s + g.size * (g.files.length - 1), 0);
      }
      selected.clear();
      showStatus(status, `${trashed} archivos enviados a la papelera.`, "ok");
    } catch (e) {
      showStatus(status, escapeHtml(String(e)), "error");
    }
    renderGroups();
  }

  el("folders").addEventListener("click", (event) => {
    const button = (event.target as HTMLElement).closest<HTMLButtonElement>("[data-remove]");
    if (!button) return;
    folders.splice(Number(button.dataset.remove), 1);
    renderFolders();
  });

  btnAdd.addEventListener("click", async () => {
    const picked = await open({ directory: true, multiple: true, title: "Carpetas en las que buscar duplicados" });
    const paths = picked === null ? [] : Array.isArray(picked) ? picked : [picked];
    for (const p of paths) if (!folders.some((f) => f.toLowerCase() === p.toLowerCase())) folders.push(p);
    renderFolders();
  });

  list.addEventListener("change", (event) => {
    const input = event.target as HTMLInputElement;
    const group = result?.groups[Number(input.dataset.group)];
    const isFull = () => group?.files.every((f) => selected.has(f.path));
    const wasFull = isFull();
    if (input.checked) selected.add(input.value);
    else selected.delete(input.value);
    // Solo se vuelve a pintar si aparece o desaparece el aviso de "todas las copias".
    if (isFull() !== wasFull) renderGroups();
    else updateSummary();
  });

  btnSearch.addEventListener("click", search);
  btnCancel.addEventListener("click", () => invoke("cancel_duplicates"));
  btnOldest.addEventListener("click", keepOldest);
  btnTrash.addEventListener("click", trash);
  listen<Progress>("duplicates-progress", (event) => {
    if (scanning) showProgress(event.payload);
  });

  renderFolders();
  invoke<string[]>("default_duplicate_folders")
    .then((defaults) => {
      folders = defaults;
      renderFolders();
    })
    .catch((e) => showStatus(status, `No se pudieron obtener las carpetas por defecto: ${escapeHtml(String(e))}`, "error"));
}

export const duplicateFinder: Module = {
  id: "duplicate-finder",
  label: "Duplicate Finder",
  description: "Archivos con el mismo contenido en tus carpetas. Se envían a la papelera.",
  // Dos documentos superpuestos.
  icon: icon(
    `<rect x="8.5" y="8.5" width="11.5" height="11.5" rx="2"/>
     <path d="M15.5 8.5V6a2 2 0 0 0-2-2H6a2 2 0 0 0-2 2v7.5a2 2 0 0 0 2 2h2.5"/>`,
  ),
  render,
};
