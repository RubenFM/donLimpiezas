import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { open } from "@tauri-apps/plugin-dialog";
import { escapeHtml, formatBytes, showStatus } from "../ui";
import { icon, type Module } from "./module";

type Progress = { files: number; dirs: number; current: string };
type Summary = { root: string; size: number; files: number; dirs: number; unreadable: number; secs: number };
type Item = { name: string; kind: "dir" | "file" | "other"; path: string | null; size: number; count: number | null };
type Folder = { path: string; size: number; items: Item[] };
type SystemInfo = { disks: { letter: string; free: number; total: number }[] };

const CUSTOM = "__custom__";

const template = `
  <div class="toolbar">
    <select data-el="target" class="input" aria-label="Qué analizar"></select>
    <button data-el="analyze" class="btn primary">Analizar</button>
    <button data-el="cancel" class="btn" hidden>Cancelar</button>
    <button data-el="explore" class="btn" disabled>Abrir en el Explorador</button>
    <span data-el="summary" class="summary"></span>
  </div>
  <div data-el="progress" class="progress" hidden><div class="progress-bar indeterminate"></div><span></span></div>
  <nav data-el="crumbs" class="crumbs" aria-label="Ruta"></nav>
  <div data-el="list" class="list">
    <p class="empty">Elige un disco o una carpeta y pulsa <strong>Analizar</strong>. No se borra nada.</p>
  </div>
  <div data-el="status" class="status" hidden></div>`;

const ICONS = {
  dir: icon(`<path d="M3.5 7.5a2 2 0 0 1 2-2h4l2 2h7a2 2 0 0 1 2 2v8a2 2 0 0 1-2 2h-13a2 2 0 0 1-2-2z"/>`),
  file: icon(`<path d="M14 3.5H7a2 2 0 0 0-2 2v13a2 2 0 0 0 2 2h10a2 2 0 0 0 2-2V8.5z"/><path d="M14 3.5v5h5"/>`),
  other: icon(`<circle cx="6" cy="12" r="1.2"/><circle cx="12" cy="12" r="1.2"/><circle cx="18" cy="12" r="1.2"/>`),
};

function render(root: HTMLElement) {
  root.insertAdjacentHTML("beforeend", template);
  const el = <T extends HTMLElement>(name: string) => root.querySelector(`[data-el="${name}"]`) as T;
  const list = el("list");
  const status = el("status");
  const progress = el("progress");
  const target = el<HTMLSelectElement>("target");
  const btnAnalyze = el<HTMLButtonElement>("analyze");
  const btnCancel = el<HTMLButtonElement>("cancel");
  const btnExplore = el<HTMLButtonElement>("explore");

  let summary: Summary | null = null;
  /** Carpetas desde la raíz analizada hasta la actual. */
  let trail: string[] = [];
  let scanning = false;

  function renderCrumbs() {
    const crumbs = el("crumbs");
    crumbs.innerHTML = trail
      .map((path, i) => {
        const name = i === 0 ? path : path.slice(path.lastIndexOf("\\") + 1);
        return i === trail.length - 1
          ? `<span aria-current="page">${escapeHtml(name)}</span>`
          : `<button class="crumb" data-crumb="${i}">${escapeHtml(name)}</button><span class="crumb-sep">›</span>`;
      })
      .join("");
  }

  async function show(path: string) {
    try {
      const folder = await invoke<Folder>("disk_children", { path });
      list.innerHTML = folder.items.length
        ? folder.items
            .map((item) => {
              const pct = folder.size ? (100 * item.size) / folder.size : 0;
              const name = item.kind === "other" ? `${item.count} archivos más` : item.name;
              const body = `
                <span class="disk-icon ${item.kind}">${ICONS[item.kind]}</span>
                <div class="row-main">
                  <div class="row-title">${escapeHtml(name)}</div>
                  <div class="meter"><div style="width: ${pct.toFixed(1)}%"></div></div>
                </div>
                <div class="row-size"><strong>${formatBytes(item.size)}</strong><span>${pct.toFixed(1)} %</span></div>`;
              return item.kind === "dir"
                ? `<button class="row disk-row" data-path="${escapeHtml(item.path!)}">${body}</button>`
                : `<div class="row disk-row">${body}</div>`;
            })
            .join("")
        : '<p class="empty">Carpeta vacía.</p>';
      list.scrollTop = 0;
      renderCrumbs();
    } catch (e) {
      showStatus(status, escapeHtml(String(e)), "error");
    }
  }

  function setScanning(on: boolean) {
    scanning = on;
    btnAnalyze.hidden = on;
    btnCancel.hidden = !on;
    target.disabled = on;
    btnExplore.disabled = on || trail.length === 0;
    progress.hidden = !on;
  }

  async function analyze() {
    let path = target.value;
    if (path === CUSTOM) {
      const picked = await open({ directory: true, title: "Carpeta que analizar" });
      if (typeof picked !== "string") return;
      path = picked;
    }
    status.hidden = true;
    list.innerHTML = "";
    trail = [];
    renderCrumbs();
    el("summary").textContent = "";
    setScanning(true);
    progress.querySelector("span")!.textContent = "Empezando…";
    try {
      summary = await invoke<Summary>("analyze_disk", { path });
      trail = [summary.root];
      el("summary").textContent =
        `${formatBytes(summary.size)} en ${summary.files.toLocaleString("es-ES")} archivos · ` +
        `${summary.secs.toLocaleString("es-ES", { maximumFractionDigits: 1 })} s` +
        (summary.unreadable ? ` · ${summary.unreadable} carpetas sin permiso` : "");
      await show(summary.root);
    } catch (e) {
      showStatus(status, escapeHtml(String(e)), "info");
    } finally {
      setScanning(false);
    }
  }

  list.addEventListener("click", (event) => {
    const row = (event.target as HTMLElement).closest<HTMLButtonElement>("[data-path]");
    if (!row) return;
    trail.push(row.dataset.path!);
    show(row.dataset.path!);
  });
  el("crumbs").addEventListener("click", (event) => {
    const crumb = (event.target as HTMLElement).closest<HTMLButtonElement>("[data-crumb]");
    if (!crumb) return;
    trail = trail.slice(0, Number(crumb.dataset.crumb) + 1);
    show(trail[trail.length - 1]);
  });
  btnAnalyze.addEventListener("click", analyze);
  btnCancel.addEventListener("click", () => invoke("cancel_disk_analysis"));
  btnExplore.addEventListener("click", () =>
    invoke("open_in_explorer", { path: trail[trail.length - 1] }).catch((e) => showStatus(status, escapeHtml(String(e)), "error")),
  );
  listen<Progress>("disk-progress", (event) => {
    if (!scanning) return;
    const p = event.payload;
    progress.querySelector("span")!.textContent =
      `${p.files.toLocaleString("es-ES")} archivos · ${p.dirs.toLocaleString("es-ES")} carpetas · ${p.current}`;
  });

  invoke<SystemInfo>("system_info")
    .then((info) => info.disks)
    .catch(() => [])
    .then((disks) => {
      target.innerHTML =
        disks
          .map(
            (d) =>
              `<option value="${escapeHtml(d.letter)}\\">Disco ${escapeHtml(d.letter)} (${formatBytes(d.total - d.free)} usados)</option>`,
          )
          .join("") + `<option value="${CUSTOM}">Elegir carpeta…</option>`;
    });
}

export const diskAnalyzer: Module = {
  id: "disk-analyzer",
  label: "Disk Analyzer",
  description: "Qué carpetas y archivos ocupan tu disco. Solo analiza; no borra nada.",
  // Gráfico circular.
  icon: icon(
    `<path d="M12 3.5a8.5 8.5 0 1 0 8.5 8.5H12z"/>
     <path d="M15 3.9A8.5 8.5 0 0 1 20.1 9H15z"/>`,
  ),
  render,
};
