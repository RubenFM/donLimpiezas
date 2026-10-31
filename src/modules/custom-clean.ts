import { invoke } from "@tauri-apps/api/core";
import { app } from "../app";
import { confirmDialog, escapeHtml, formatBytes, groupBy, setBusy, showStatus } from "../ui";
import { icon, type Module } from "./module";

type CategoryScan = {
  id: string;
  name: string;
  description: string;
  group: string;
  admin: boolean;
  defaultSelected: boolean;
  warning: string | null;
  running: boolean;
  sizeKnown: boolean;
  bytes: number;
  files: number;
};

type CleanResult = { id: string; freed: number; deleted: number; failed: number };

const template = `
  <div class="toolbar">
    <button data-el="scan" class="btn">Analizar</button>
    <button data-el="clean" class="btn primary" disabled>Limpiar seleccionados</button>
    <span data-el="summary" class="summary"></span>
  </div>
  <div data-el="list" class="list">
    <p class="empty">Pulsa <strong>Analizar</strong> para ver cuánto espacio puedes liberar.</p>
  </div>
  <div data-el="status" class="status" hidden></div>`;

function render(root: HTMLElement) {
  root.insertAdjacentHTML("beforeend", template);
  const el = <T extends HTMLElement>(name: string) => root.querySelector(`[data-el="${name}"]`) as T;
  const list = el("list");
  const status = el("status");
  const btnScan = el<HTMLButtonElement>("scan");
  const btnClean = el<HTMLButtonElement>("clean");

  let categories: CategoryScan[] = [];

  function selectedIds(): string[] {
    return [...list.querySelectorAll<HTMLInputElement>("input[type=checkbox]:checked")].map(
      (c) => c.value,
    );
  }

  function updateSummary() {
    const ids = new Set(selectedIds());
    const total = categories.filter((c) => ids.has(c.id)).reduce((sum, c) => sum + c.bytes, 0);
    el("summary").textContent = categories.length
      ? `${formatBytes(total)} seleccionados para liberar`
      : "";
    btnClean.disabled = ids.size === 0;
  }

  function renderCategories() {
    list.innerHTML = [...groupBy(categories)]
      .map(([group, items]) => {
        const rows = items
          .map((c) => {
            const locked = c.admin && !app.isAdmin;
            const checked = !locked && c.defaultSelected && c.bytes > 0;
            const notes = [
              c.warning,
              c.running ? "Ciérralo antes para limpiarlo por completo; los archivos en uso se omitirán." : null,
            ].filter((n): n is string => n !== null);
            return `
              <label class="row ${locked ? "locked" : ""}">
                <input type="checkbox" value="${c.id}" ${checked ? "checked" : ""} ${locked ? "disabled" : ""} />
                <div class="row-main">
                  <div class="row-title">${escapeHtml(c.name)}
                    ${c.admin ? '<span class="pill">Admin</span>' : ""}
                    ${c.running ? '<span class="pill warn">Abierto</span>' : ""}</div>
                  <div class="row-desc">${escapeHtml(c.description)}</div>
                  ${notes.map((n) => `<div class="row-warning">${escapeHtml(n)}</div>`).join("")}
                </div>
                <div class="row-size">
                  <strong>${c.sizeKnown ? formatBytes(c.bytes) : "—"}</strong>
                  <span>${c.sizeKnown ? `${c.files.toLocaleString("es-ES")} elementos` : "tamaño desconocido"}</span>
                </div>
              </label>`;
          })
          .join("");
        return `<h3 class="group">${escapeHtml(group)}</h3>${rows}`;
      })
      .join("");
    updateSummary();
  }

  async function scan() {
    setBusy(btnScan, true, "Analizando…");
    btnClean.disabled = true;
    try {
      categories = await invoke<CategoryScan[]>("scan_cleanup");
      renderCategories();
    } catch (e) {
      showStatus(status, `Error al analizar: ${escapeHtml(String(e))}`, "error");
    } finally {
      setBusy(btnScan, false);
      updateSummary();
    }
  }

  async function clean() {
    const ids = selectedIds();
    const names = categories.filter((c) => ids.includes(c.id)).map((c) => escapeHtml(c.name));
    const ok = await confirmDialog(
      "¿Limpiar los elementos seleccionados?",
      `Se borrarán de forma permanente:<br>• ${names.join("<br>• ")}`,
    );
    if (!ok) return;

    const slow = categories.some((c) => ids.includes(c.id) && !c.sizeKnown);
    setBusy(btnClean, true, slow ? "Limpiando… (varios minutos)" : "Limpiando…");
    try {
      const results = await invoke<CleanResult[]>("run_cleanup", { ids });
      const freed = results.reduce((s, r) => s + r.freed, 0);
      const deleted = results.reduce((s, r) => s + r.deleted, 0);
      const failed = results.reduce((s, r) => s + r.failed, 0);
      showStatus(
        status,
        `Liberados <strong>${formatBytes(freed)}</strong> (${deleted.toLocaleString("es-ES")} elementos).` +
          (failed ? ` ${failed.toLocaleString("es-ES")} archivos estaban en uso y se han omitido.` : "") +
          (slow ? " El espacio de los componentes de Windows es aproximado." : ""),
        "ok",
      );
    } catch (e) {
      showStatus(status, `Error al limpiar: ${escapeHtml(String(e))}`, "error");
    } finally {
      setBusy(btnClean, false);
    }
    await scan();
  }

  list.addEventListener("change", updateSummary);
  btnScan.addEventListener("click", scan);
  btnClean.addEventListener("click", clean);
}

export const customClean: Module = {
  id: "custom-clean",
  label: "Custom Clean",
  description: "Temporales, cachés, papelera y restos del sistema.",
  // Escoba: palo en diagonal y cepillo abajo a la izquierda.
  icon: icon(
    `<path d="M20 4 13.5 10.5"/>
     <path d="M10.5 8.5l5 5"/>
     <path d="M10.5 8.5C8.5 10 5.5 10.8 3 10.8c0 4.2 3 8.4 6.2 10.2 2.6-1.6 4.6-4.4 6.3-7.5"/>
     <path d="M6.2 15.2 9 18"/>`,
  ),
  render,
};
