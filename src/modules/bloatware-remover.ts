import { invoke } from "@tauri-apps/api/core";
import { app } from "../app";
import { confirmDialog, escapeHtml, groupBy, setBusy, showStatus } from "../ui";
import { icon, type Module } from "./module";

type BloatApp = {
  id: string;
  name: string;
  group: string;
  note: string;
  installed: boolean;
  provisioned: boolean;
};

type RemoveResult = { name: string; ok: boolean; error: string | null };

const template = `
  <div class="toolbar">
    <button data-el="search" class="btn">Buscar apps</button>
    <button data-el="remove" class="btn danger" disabled>Quitar seleccionadas</button>
    <label class="option"><input type="checkbox" data-el="restore" /> Crear punto de restauración</label>
    <label class="option"><input type="checkbox" data-el="all-users" /> Para todos los usuarios</label>
  </div>
  <div data-el="list" class="list">
    <p class="empty">Pulsa <strong>Buscar apps</strong> para detectar el bloatware instalado.</p>
  </div>
  <div data-el="status" class="status" hidden></div>`;

function render(root: HTMLElement) {
  root.insertAdjacentHTML("beforeend", template);
  const el = <T extends HTMLElement>(name: string) => root.querySelector(`[data-el="${name}"]`) as T;
  const list = el("list");
  const status = el("status");
  const btnSearch = el<HTMLButtonElement>("search");
  const btnRemove = el<HTMLButtonElement>("remove");
  const optRestore = el<HTMLInputElement>("restore");
  const optAllUsers = el<HTMLInputElement>("all-users");

  optRestore.disabled = !app.isAdmin;
  optAllUsers.disabled = !app.isAdmin;
  optRestore.checked = app.isAdmin;

  let apps: BloatApp[] = [];

  function selectedIds(): string[] {
    return [...list.querySelectorAll<HTMLInputElement>("input[type=checkbox]:checked")].map(
      (c) => c.value,
    );
  }

  function updateSummary() {
    const n = selectedIds().length;
    btnRemove.disabled = n === 0;
    btnRemove.textContent = n ? `Quitar ${n} seleccionada${n === 1 ? "" : "s"}` : "Quitar seleccionadas";
  }

  function renderApps() {
    const present = apps.filter((a) => a.installed || a.provisioned);
    if (present.length === 0) {
      list.innerHTML = '<p class="empty">No se ha encontrado bloatware del catálogo. 🎉</p>';
      updateSummary();
      return;
    }
    list.innerHTML = [...groupBy(present)]
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
    updateSummary();
  }

  async function search() {
    setBusy(btnSearch, true, "Buscando…");
    try {
      apps = await invoke<BloatApp[]>("list_bloat");
      renderApps();
    } catch (e) {
      showStatus(status, `Error al listar apps: ${escapeHtml(String(e))}`, "error");
    } finally {
      setBusy(btnSearch, false);
    }
  }

  async function remove() {
    const ids = selectedIds();
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
        showStatus(status, "Creando punto de restauración…");
        try {
          messages.push(escapeHtml(await invoke<string>("create_restore_point", { purpose: "apps" })));
        } catch (e) {
          const go = await confirmDialog(
            "No se pudo crear el punto de restauración",
            `${escapeHtml(String(e))}<br><br>¿Quieres continuar igualmente?`,
          );
          if (!go) {
            status.hidden = true;
            return;
          }
        }
      }

      showStatus(status, "Quitando apps…");
      const results = await invoke<RemoveResult[]>("remove_bloat", {
        ids,
        allUsers: optAllUsers.checked,
      });
      const okCount = results.filter((r) => r.ok).length;
      messages.push(`${okCount} de ${results.length} apps quitadas.`);
      for (const r of results.filter((r) => !r.ok)) {
        messages.push(`✗ <code>${escapeHtml(r.name)}</code>: ${escapeHtml(r.error ?? "error desconocido")}`);
      }
      showStatus(status, messages.join("<br>"), okCount === results.length ? "ok" : "error");
    } catch (e) {
      showStatus(status, `Error al quitar apps: ${escapeHtml(String(e))}`, "error");
    } finally {
      setBusy(btnRemove, false);
    }
    await search();
  }

  list.addEventListener("change", updateSummary);
  btnSearch.addEventListener("click", search);
  btnRemove.addEventListener("click", remove);
}

export const bloatwareRemover: Module = {
  id: "bloatware-remover",
  label: "Bloatware Remover",
  description: "Apps preinstaladas de Windows que no necesitas.",
  // Caja de paquete precintada.
  icon: icon(
    `<path d="M20 7.5 12 3 4 7.5v9L12 21l8-4.5z"/>
     <path d="M4 7.5 12 12l8-4.5"/>
     <path d="M12 12v9"/>
     <path d="M7.5 5.5 15.5 10"/>`,
  ),
  render,
};
