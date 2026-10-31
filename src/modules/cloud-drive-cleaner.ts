import { invoke } from "@tauri-apps/api/core";
import { confirmDialog, escapeHtml, formatBytes, setBusy, showStatus } from "../ui";
import { icon, type Module } from "./module";

type CloudRoot = { name: string; path: string };
type CloudFile = { path: string; root: string; size: number; modified: number | null; pinned: boolean };
type CloudScan = { files: CloudFile[]; totalFiles: number; matchingBytes: number; localBytes: number };
type FreeResult = { freed: number; bytes: number; failed: string[] };

const template = `
  <div class="toolbar">
    <select data-el="root" class="input" aria-label="Carpeta"></select>
    <select data-el="min" class="input" aria-label="Tamaño mínimo">
      <option value="1048576">≥ 1 MB</option>
      <option value="10485760" selected>≥ 10 MB</option>
      <option value="104857600">≥ 100 MB</option>
    </select>
    <select data-el="age" class="input" aria-label="Antigüedad">
      <option value="0" selected>Cualquier fecha</option>
      <option value="30">Sin cambios en 30 días</option>
      <option value="90">Sin cambios en 90 días</option>
      <option value="365">Sin cambios en un año</option>
    </select>
    <button data-el="scan" class="btn">Analizar</button>
  </div>
  <div class="toolbar">
    <button data-el="all" class="btn" disabled>Seleccionar todo</button>
    <button data-el="free" class="btn primary" disabled>Liberar espacio</button>
    <span data-el="summary" class="summary"></span>
  </div>
  <div data-el="list" class="list">
    <p class="empty">Buscando carpetas de OneDrive y Dropbox…</p>
  </div>
  <div data-el="status" class="status" hidden></div>`;

function splitPath(path: string): [string, string] {
  const i = path.lastIndexOf("\\");
  return [path.slice(0, i), path.slice(i + 1)];
}

function render(root: HTMLElement) {
  root.insertAdjacentHTML("beforeend", template);
  const el = <T extends HTMLElement>(name: string) => root.querySelector(`[data-el="${name}"]`) as T;
  const list = el("list");
  const status = el("status");
  const rootSelect = el<HTMLSelectElement>("root");
  const btnScan = el<HTMLButtonElement>("scan");
  const btnAll = el<HTMLButtonElement>("all");
  const btnFree = el<HTMLButtonElement>("free");

  let roots: CloudRoot[] = [];
  let scan: CloudScan | null = null;
  const selected = new Set<string>();

  function selectableFiles(): CloudFile[] {
    return scan?.files.filter((f) => !f.pinned) ?? [];
  }

  function updateSummary() {
    if (!scan) {
      el("summary").textContent = "";
      btnAll.disabled = btnFree.disabled = true;
      return;
    }
    const chosen = scan.files.filter((f) => selected.has(f.path)).reduce((s, f) => s + f.size, 0);
    el("summary").textContent =
      `${formatBytes(scan.localBytes)} descargados en este equipo · ` +
      `${scan.totalFiles} archivos (${formatBytes(scan.matchingBytes)}) cumplen los filtros` +
      (selected.size ? ` · ${formatBytes(chosen)} seleccionados` : "");
    btnFree.disabled = selected.size === 0;
    const all = selectableFiles();
    btnAll.disabled = all.length === 0;
    btnAll.textContent = all.length && all.every((f) => selected.has(f.path)) ? "Quitar selección" : "Seleccionar todo";
  }

  function renderFiles() {
    if (!scan) return;
    if (scan.files.length === 0) {
      list.innerHTML = '<p class="empty">No hay archivos descargados que cumplan los filtros. 🎉</p>';
      updateSummary();
      return;
    }
    const more =
      scan.totalFiles > scan.files.length
        ? `<p class="row-note list-note">Mostrando los ${scan.files.length} más grandes de ${scan.totalFiles}.</p>`
        : "";
    list.innerHTML =
      scan.files
        .map((f) => {
          const [dir, name] = splitPath(f.path);
          const date = f.modified ? new Date(f.modified * 1000).toLocaleDateString("es-ES") : "—";
          return `
            <label class="row">
              <input type="checkbox" value="${escapeHtml(f.path)}" ${selected.has(f.path) ? "checked" : ""} />
              <div class="row-main">
                <div class="row-title">${escapeHtml(name)}
                  ${f.pinned ? '<span class="pill" title="Marcado como «Mantener siempre en este dispositivo»">Fijado</span>' : ""}</div>
                <div class="row-desc" title="${escapeHtml(f.path)}">${escapeHtml(dir)}</div>
              </div>
              <div class="row-size"><strong>${formatBytes(f.size)}</strong><span>${date}</span></div>
            </label>`;
        })
        .join("") + more;
    updateSummary();
  }

  async function analyze() {
    const paths = rootSelect.value ? [rootSelect.value] : roots.map((r) => r.path);
    setBusy(btnScan, true, "Analizando…");
    status.hidden = true;
    selected.clear();
    try {
      scan = await invoke<CloudScan>("scan_cloud", {
        roots: paths,
        minSize: Number(el<HTMLSelectElement>("min").value),
        minAgeDays: Number(el<HTMLSelectElement>("age").value),
      });
      renderFiles();
    } catch (e) {
      showStatus(status, escapeHtml(String(e)), "error");
    } finally {
      setBusy(btnScan, false);
    }
  }

  async function free() {
    const paths = [...selected];
    const pinned = scan?.files.filter((f) => f.pinned && selected.has(f.path)).length ?? 0;
    const ok = await confirmDialog(
      "¿Dejar estos archivos solo en la nube?",
      `${paths.length} archivos dejarán de ocupar espacio en este equipo. <strong>No se borran</strong>: ` +
        "siguen en la nube y se descargan al abrirlos, pero necesitarás conexión para ello." +
        (pinned ? `<br><br>${pinned} estaban marcados para mantenerse siempre en este dispositivo.` : ""),
    );
    if (!ok) return;
    setBusy(btnFree, true, "Liberando…");
    try {
      const result = await invoke<FreeResult>("free_cloud_files", { paths });
      const lines = [`${result.freed} archivos liberados (${formatBytes(result.bytes)}).`];
      if (result.failed.length) {
        lines.push(`${result.failed.length} no se pudieron liberar (puede que estén pendientes de subir o abiertos):`);
        lines.push(...result.failed.slice(0, 5).map((p) => `• ${escapeHtml(p)}`));
      }
      showStatus(status, lines.join("<br>"), result.failed.length ? "error" : "ok");
      if (scan) {
        const gone = new Set(paths.filter((p) => !result.failed.includes(p)));
        scan.files = scan.files.filter((f) => !gone.has(f.path));
        scan.totalFiles -= gone.size;
        scan.matchingBytes -= result.bytes;
        scan.localBytes -= result.bytes;
      }
      selected.clear();
      renderFiles();
    } catch (e) {
      showStatus(status, escapeHtml(String(e)), "error");
    } finally {
      setBusy(btnFree, false);
      updateSummary();
    }
  }

  list.addEventListener("change", (event) => {
    const input = event.target as HTMLInputElement;
    if (input.checked) selected.add(input.value);
    else selected.delete(input.value);
    updateSummary();
  });
  btnAll.addEventListener("click", () => {
    const all = selectableFiles();
    const allSelected = all.every((f) => selected.has(f.path));
    selected.clear();
    if (!allSelected) for (const f of all) selected.add(f.path);
    renderFiles();
  });
  btnScan.addEventListener("click", analyze);
  btnFree.addEventListener("click", free);

  invoke<CloudRoot[]>("cloud_roots").then((found) => {
    roots = found;
    if (roots.length === 0) {
      list.innerHTML = '<p class="empty">No se ha encontrado OneDrive ni Dropbox en este equipo.</p>';
      btnScan.disabled = true;
      return;
    }
    rootSelect.innerHTML =
      (roots.length > 1 ? '<option value="">Todas las carpetas</option>' : "") +
      roots.map((r) => `<option value="${escapeHtml(r.path)}">${escapeHtml(r.name)}</option>`).join("");
    analyze();
  });
}

export const cloudDriveCleaner: Module = {
  id: "cloud-drive-cleaner",
  label: "Cloud Drive Cleaner",
  description: "Deja solo en la nube archivos de OneDrive y Dropbox para liberar espacio sin borrarlos.",
  // Nube con flecha hacia arriba.
  icon: icon(
    `<path d="M7 18.5a4.5 4.5 0 0 1-.6-9A6 6 0 0 1 18 10a4 4 0 0 1-.5 8.5"/>
     <path d="M12 19.5V12.5"/>
     <path d="M9.5 15l2.5-2.5 2.5 2.5"/>`,
  ),
  render,
};
