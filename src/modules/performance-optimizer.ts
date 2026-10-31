import { invoke } from "@tauri-apps/api/core";
import { app } from "../app";
import { confirmDialog, escapeHtml, groupBy, setBusy, showStatus } from "../ui";
import { icon, type Module } from "./module";

type Tweak = {
  id: string;
  name: string;
  description: string;
  group: string;
  note: string | null;
  admin: boolean;
  applied: boolean;
  restorable: boolean;
};

type TweakResult = { id: string; ok: boolean; error: string | null };

const template = `
  <div class="toolbar">
    <button data-el="apply" class="btn primary" disabled>Aplicar seleccionados</button>
    <button data-el="revert-all" class="btn" disabled>Restaurar todo</button>
    <span data-el="summary" class="summary"></span>
  </div>
  <div data-el="list" class="list">
    <p class="empty">Cargando ajustes…</p>
  </div>
  <div data-el="status" class="status" hidden></div>`;

function render(root: HTMLElement) {
  root.insertAdjacentHTML("beforeend", template);
  const el = <T extends HTMLElement>(name: string) => root.querySelector(`[data-el="${name}"]`) as T;
  const list = el("list");
  const status = el("status");
  const btnApply = el<HTMLButtonElement>("apply");
  const btnRevertAll = el<HTMLButtonElement>("revert-all");

  let tweaks: Tweak[] = [];
  const locked = (t: Tweak) => t.admin && !app.isAdmin;

  function selectedIds(): string[] {
    return [...list.querySelectorAll<HTMLInputElement>("input[type=checkbox]:checked")].map((c) => c.value);
  }

  function updateSummary() {
    const applied = tweaks.filter((t) => t.applied).length;
    el("summary").textContent = tweaks.length ? `${applied} de ${tweaks.length} aplicados` : "";
    btnApply.disabled = selectedIds().length === 0;
    btnRevertAll.disabled = !tweaks.some((t) => t.restorable && !locked(t));
  }

  function renderTweaks() {
    list.innerHTML = [...groupBy(tweaks)]
      .map(([group, items]) => {
        const rows = items
          .map((t) => {
            const pills = [
              t.admin ? '<span class="pill">Admin</span>' : "",
              t.applied ? '<span class="pill ok">Aplicado</span>' : "",
            ].join(" ");
            const body = `
              <div class="row-main">
                <div class="row-title">${escapeHtml(t.name)} ${pills}</div>
                <div class="row-desc">${escapeHtml(t.description)}</div>
                ${t.note ? `<div class="row-note">${escapeHtml(t.note)}</div>` : ""}
              </div>`;
            if (!t.applied) {
              return `
                <label class="row ${locked(t) ? "locked" : ""}">
                  <input type="checkbox" value="${t.id}" ${locked(t) ? "disabled" : "checked"} />
                  ${body}
                </label>`;
            }
            const action = t.restorable
              ? `<button class="btn small" data-revert="${t.id}" ${locked(t) ? "disabled" : ""}>Restaurar</button>`
              : '<span class="row-note">Ya estaba así</span>';
            return `<div class="row"><span class="row-spacer"></span>${body}${action}</div>`;
          })
          .join("");
        return `<h3 class="group">${escapeHtml(group)}</h3>${rows}`;
      })
      .join("");
    updateSummary();
  }

  async function load() {
    try {
      tweaks = await invoke<Tweak[]>("list_tweaks");
      renderTweaks();
    } catch (e) {
      showStatus(status, `Error al leer los ajustes: ${escapeHtml(String(e))}`, "error");
    }
  }

  function report(results: TweakResult[], verb: string) {
    const name = (id: string) => escapeHtml(tweaks.find((t) => t.id === id)?.name ?? id);
    const okCount = results.filter((r) => r.ok).length;
    const lines = [`${okCount} de ${results.length} ajustes ${verb}.`];
    for (const r of results.filter((r) => !r.ok)) lines.push(`✗ ${name(r.id)}: ${escapeHtml(r.error ?? "")}`);
    const pending = results.filter((r) => r.ok && tweaks.find((t) => t.id === r.id)?.note);
    if (pending.length) lines.push("Algunos cambios surten efecto al reiniciar o cerrar sesión.");
    showStatus(status, lines.join("<br>"), okCount === results.length ? "ok" : "error");
  }

  async function run(command: "apply_tweaks" | "revert_tweaks", ids: string[], button: HTMLButtonElement, verb: string) {
    setBusy(button, true, "Aplicando…");
    try {
      report(await invoke<TweakResult[]>(command, { ids }), verb);
    } catch (e) {
      showStatus(status, escapeHtml(String(e)), "error");
    } finally {
      setBusy(button, false);
    }
    await load();
  }

  btnApply.addEventListener("click", async () => {
    const ids = selectedIds();
    const names = tweaks.filter((t) => ids.includes(t.id)).map((t) => escapeHtml(t.name));
    const ok = await confirmDialog(
      "¿Aplicar estos ajustes?",
      `• ${names.join("<br>• ")}<br><br>Se guardan los valores actuales para poder restaurarlos.`,
    );
    if (ok) await run("apply_tweaks", ids, btnApply, "aplicados");
  });

  btnRevertAll.addEventListener("click", async () => {
    const ids = tweaks.filter((t) => t.restorable && !locked(t)).map((t) => t.id);
    const ok = await confirmDialog(
      "¿Restaurar todos los ajustes?",
      `Se devolverán ${ids.length} ajustes a los valores que tenían antes de aplicarlos.`,
    );
    if (ok) await run("revert_tweaks", ids, btnRevertAll, "restaurados");
  });

  // Restaurar uno: sin confirmación, se puede volver a aplicar.
  list.addEventListener("click", (event) => {
    const button = (event.target as HTMLElement).closest<HTMLButtonElement>("[data-revert]");
    if (button) run("revert_tweaks", [button.dataset.revert!], button, "restaurados");
  });
  list.addEventListener("change", updateSummary);
  load();
}

export const performanceOptimizer: Module = {
  id: "performance-optimizer",
  label: "Performance Optimizer",
  description: "Telemetría, anuncios y funciones innecesarias de Windows, siempre reversibles.",
  // Velocímetro.
  icon: icon(
    `<path d="M4 17.5a9 9 0 1 1 16 0"/>
     <path d="M12 13.5l4.5-4.5"/>
     <circle cx="12" cy="13.5" r="1.2"/>`,
  ),
  render,
};
