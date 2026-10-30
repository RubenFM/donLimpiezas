import { invoke } from "@tauri-apps/api/core";

type CategoryScan = {
  id: string;
  name: string;
  description: string;
  group: string;
  admin: boolean;
  bytes: number;
  files: number;
};

type CleanResult = { id: string; freed: number; deleted: number; failed: number };

type BloatApp = {
  id: string;
  name: string;
  group: string;
  note: string;
  installed: boolean;
  provisioned: boolean;
};

type RemoveResult = { name: string; ok: boolean; error: string | null };

const $ = <T extends HTMLElement>(sel: string) => document.querySelector(sel) as T;

let isAdmin = false;
let categories: CategoryScan[] = [];
let apps: BloatApp[] = [];

function formatBytes(n: number): string {
  if (n < 1024) return `${n} B`;
  const units = ["KB", "MB", "GB", "TB"];
  let value = n / 1024;
  let i = 0;
  while (value >= 1024 && i < units.length - 1) {
    value /= 1024;
    i++;
  }
  return `${value.toFixed(value < 10 ? 1 : 0)} ${units[i]}`;
}

function escapeHtml(s: string): string {
  return s.replace(/[&<>"']/g, (c) => `&#${c.charCodeAt(0)};`);
}

function groupBy<T extends { group: string }>(items: T[]): Map<string, T[]> {
  const groups = new Map<string, T[]>();
  for (const item of items) {
    if (!groups.has(item.group)) groups.set(item.group, []);
    groups.get(item.group)!.push(item);
  }
  return groups;
}

function showStatus(el: HTMLElement, html: string, kind: "ok" | "error" | "info" = "info") {
  el.hidden = false;
  el.className = `status ${kind}`;
  el.innerHTML = html;
}

function setBusy(button: HTMLButtonElement, busy: boolean, label?: string) {
  if (busy) {
    button.dataset.label = button.textContent ?? "";
    button.textContent = label ?? "Trabajando…";
  } else if (button.dataset.label) {
    button.textContent = button.dataset.label;
  }
  button.disabled = busy;
  document.body.classList.toggle("busy", busy);
}

function confirmDialog(title: string, body: string): Promise<boolean> {
  const dialog = $<HTMLDialogElement>("#confirm");
  $("#confirm-title").textContent = title;
  $("#confirm-body").innerHTML = body;
  dialog.returnValue = "";
  dialog.showModal();
  return new Promise((resolve) => {
    dialog.addEventListener("close", () => resolve(dialog.returnValue === "ok"), { once: true });
  });
}

// ---------- Pestañas ----------

document.querySelectorAll<HTMLButtonElement>(".tab").forEach((tab) => {
  tab.addEventListener("click", () => {
    document.querySelectorAll(".tab").forEach((t) => t.classList.toggle("active", t === tab));
    document
      .querySelectorAll(".panel")
      .forEach((p) => p.classList.toggle("active", p.id === `tab-${tab.dataset.tab}`));
  });
});

// ---------- Limpieza ----------

const cleanList = $("#clean-list");
const cleanStatus = $("#clean-status");
const btnScan = $<HTMLButtonElement>("#btn-scan");
const btnClean = $<HTMLButtonElement>("#btn-clean");

function selectedCategoryIds(): string[] {
  return [...cleanList.querySelectorAll<HTMLInputElement>("input[type=checkbox]:checked")].map(
    (c) => c.value,
  );
}

function updateCleanSummary() {
  const ids = new Set(selectedCategoryIds());
  const total = categories.filter((c) => ids.has(c.id)).reduce((sum, c) => sum + c.bytes, 0);
  $("#clean-summary").textContent = categories.length
    ? `${formatBytes(total)} seleccionados para liberar`
    : "";
  btnClean.disabled = ids.size === 0;
}

function renderCategories() {
  const html = [...groupBy(categories)]
    .map(([group, items]) => {
      const rows = items
        .map((c) => {
          const locked = c.admin && !isAdmin;
          const checked = !locked && c.bytes > 0;
          return `
            <label class="row ${locked ? "locked" : ""}">
              <input type="checkbox" value="${c.id}" ${checked ? "checked" : ""} ${locked ? "disabled" : ""} />
              <div class="row-main">
                <div class="row-title">${escapeHtml(c.name)}
                  ${c.admin ? '<span class="pill">Admin</span>' : ""}</div>
                <div class="row-desc">${escapeHtml(c.description)}</div>
              </div>
              <div class="row-size">
                <strong>${formatBytes(c.bytes)}</strong>
                <span>${c.files.toLocaleString("es-ES")} elementos</span>
              </div>
            </label>`;
        })
        .join("");
      return `<h3 class="group">${escapeHtml(group)}</h3>${rows}`;
    })
    .join("");
  cleanList.innerHTML = html;
  updateCleanSummary();
}

cleanList.addEventListener("change", updateCleanSummary);

async function scan() {
  setBusy(btnScan, true, "Analizando…");
  btnClean.disabled = true;
  try {
    categories = await invoke<CategoryScan[]>("scan_cleanup");
    renderCategories();
  } catch (e) {
    showStatus(cleanStatus, `Error al analizar: ${escapeHtml(String(e))}`, "error");
  } finally {
    setBusy(btnScan, false);
    updateCleanSummary();
  }
}

async function clean() {
  const ids = selectedCategoryIds();
  const names = categories.filter((c) => ids.includes(c.id)).map((c) => escapeHtml(c.name));
  const ok = await confirmDialog(
    "¿Limpiar los elementos seleccionados?",
    `Se borrarán de forma permanente:<br>• ${names.join("<br>• ")}`,
  );
  if (!ok) return;

  setBusy(btnClean, true, "Limpiando…");
  try {
    const results = await invoke<CleanResult[]>("run_cleanup", { ids });
    const freed = results.reduce((s, r) => s + r.freed, 0);
    const deleted = results.reduce((s, r) => s + r.deleted, 0);
    const failed = results.reduce((s, r) => s + r.failed, 0);
    showStatus(
      cleanStatus,
      `Liberados <strong>${formatBytes(freed)}</strong> (${deleted.toLocaleString("es-ES")} elementos).` +
        (failed ? ` ${failed.toLocaleString("es-ES")} archivos estaban en uso y se han omitido.` : ""),
      "ok",
    );
  } catch (e) {
    showStatus(cleanStatus, `Error al limpiar: ${escapeHtml(String(e))}`, "error");
  } finally {
    setBusy(btnClean, false);
  }
  await scan();
}

btnScan.addEventListener("click", scan);
btnClean.addEventListener("click", clean);

// ---------- Apps preinstaladas ----------

const appsList = $("#apps-list");
const appsStatus = $("#apps-status");
const btnApps = $<HTMLButtonElement>("#btn-apps");
const btnRemove = $<HTMLButtonElement>("#btn-remove");
const optRestore = $<HTMLInputElement>("#opt-restore");
const optAllUsers = $<HTMLInputElement>("#opt-allusers");

function selectedAppIds(): string[] {
  return [...appsList.querySelectorAll<HTMLInputElement>("input[type=checkbox]:checked")].map(
    (c) => c.value,
  );
}

function updateAppsSummary() {
  const n = selectedAppIds().length;
  btnRemove.disabled = n === 0;
  btnRemove.textContent = n ? `Quitar ${n} seleccionada${n === 1 ? "" : "s"}` : "Quitar seleccionadas";
}

function renderApps() {
  const present = apps.filter((a) => a.installed || a.provisioned);
  if (present.length === 0) {
    appsList.innerHTML = '<p class="empty">No se ha encontrado bloatware del catálogo. 🎉</p>';
    updateAppsSummary();
    return;
  }
  appsList.innerHTML = [...groupBy(present)]
    .map(([group, items]) => {
      const rows = items
        .map((a) => {
          const state = a.installed
            ? '<span class="pill ok">Instalada</span>'
            : '<span class="pill">Solo en la imagen</span>';
          return `
            <label class="row">
              <input type="checkbox" value="${a.id}" />
              <div class="row-main">
                <div class="row-title">${escapeHtml(a.name)} ${state}</div>
                <div class="row-desc"><code>${escapeHtml(a.id)}</code>${
                  a.note ? ` · ${escapeHtml(a.note)}` : ""
                }</div>
              </div>
            </label>`;
        })
        .join("");
      return `<h3 class="group">${escapeHtml(group)}</h3>${rows}`;
    })
    .join("");
  updateAppsSummary();
}

appsList.addEventListener("change", updateAppsSummary);

async function loadApps() {
  setBusy(btnApps, true, "Buscando…");
  try {
    apps = await invoke<BloatApp[]>("list_bloat");
    renderApps();
  } catch (e) {
    showStatus(appsStatus, `Error al listar apps: ${escapeHtml(String(e))}`, "error");
  } finally {
    setBusy(btnApps, false);
  }
}

async function removeApps() {
  const ids = selectedAppIds();
  const names = apps.filter((a) => ids.includes(a.id)).map((a) => escapeHtml(a.name));
  const scope = optAllUsers.checked
    ? "para <strong>todos los usuarios</strong> y de la imagen de Windows"
    : "para tu usuario";
  const ok = await confirmDialog(
    "¿Quitar estas apps?",
    `Se desinstalarán ${scope}:<br>• ${names.join("<br>• ")}<br><br>` +
      "Podrás reinstalarlas desde Microsoft Store si las necesitas.",
  );
  if (!ok) return;

  setBusy(btnRemove, true, "Quitando…");
  const messages: string[] = [];
  try {
    if (optRestore.checked) {
      showStatus(appsStatus, "Creando punto de restauración…");
      try {
        messages.push(escapeHtml(await invoke<string>("create_restore_point")));
      } catch (e) {
        const go = await confirmDialog(
          "No se pudo crear el punto de restauración",
          `${escapeHtml(String(e))}<br><br>¿Quieres continuar igualmente?`,
        );
        if (!go) {
          appsStatus.hidden = true;
          return;
        }
      }
    }

    showStatus(appsStatus, "Quitando apps…");
    const results = await invoke<RemoveResult[]>("remove_bloat", {
      ids,
      allUsers: optAllUsers.checked,
    });
    const okCount = results.filter((r) => r.ok).length;
    messages.push(`${okCount} de ${results.length} apps quitadas.`);
    for (const r of results.filter((r) => !r.ok)) {
      messages.push(`✗ <code>${escapeHtml(r.name)}</code>: ${escapeHtml(r.error ?? "error desconocido")}`);
    }
    showStatus(appsStatus, messages.join("<br>"), okCount === results.length ? "ok" : "error");
  } catch (e) {
    showStatus(appsStatus, `Error al quitar apps: ${escapeHtml(String(e))}`, "error");
  } finally {
    setBusy(btnRemove, false);
  }
  await loadApps();
}

btnApps.addEventListener("click", loadApps);
btnRemove.addEventListener("click", removeApps);

// ---------- Inicio ----------

async function init() {
  isAdmin = await invoke<boolean>("is_admin");
  $("#admin-banner").hidden = isAdmin;
  optRestore.disabled = !isAdmin;
  optAllUsers.disabled = !isAdmin;
  optRestore.checked = isAdmin;

  $<HTMLButtonElement>("#btn-elevate").addEventListener("click", async () => {
    try {
      await invoke("relaunch_as_admin");
    } catch (e) {
      showStatus(cleanStatus, escapeHtml(String(e)), "error");
    }
  });
}

init();
