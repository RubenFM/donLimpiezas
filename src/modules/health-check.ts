import { invoke } from "@tauri-apps/api/core";
import { app } from "../app";
import { navigate } from "../nav";
import { escapeHtml, formatBytes } from "../ui";
import { getUpdates } from "../updates";
import { icon, type Module } from "./module";

type SystemInfo = {
  windows: string;
  uptimeSecs: number;
  memoryTotal: number;
  memoryUsed: number;
  disks: { letter: string; free: number; total: number }[];
};

type Finding = { value: string; detail: string; ok: boolean };

/** Umbrales a partir de los cuales un apartado se considera mejorable. */
const LIMITS = { cleanBytes: 500 * 1024 * 1024, startupEnabled: 6 };

const available = (item: { admin: boolean }) => !item.admin || app.isAdmin;

const plural = (n: number, one: string, many: string) => `${n} ${n === 1 ? one : many}`;

type Card = { id: string; title: string; module: string; load: () => Promise<Finding> };

const cards: Card[] = [
  {
    id: "clean",
    title: "Espacio liberable",
    module: "custom-clean",
    load: async () => {
      const cats = await invoke<{ bytes: number; admin: boolean; defaultSelected: boolean }[]>("scan_cleanup");
      const bytes = cats.filter((c) => c.defaultSelected && available(c)).reduce((s, c) => s + c.bytes, 0);
      return { value: formatBytes(bytes), detail: "en temporales, cachés y papelera", ok: bytes < LIMITS.cleanBytes };
    },
  },
  {
    id: "bloat",
    title: "Apps preinstaladas",
    module: "bloatware-remover",
    load: async () => {
      const apps = await invoke<{ installed: boolean }[]>("list_bloat");
      const n = apps.filter((a) => a.installed).length;
      return { value: plural(n, "app", "apps"), detail: "de Windows que puedes quitar", ok: n === 0 };
    },
  },
  {
    id: "startup",
    title: "Programas de inicio",
    module: "startup-manager",
    load: async () => {
      const entries = await invoke<{ enabled: boolean }[]>("list_startup");
      const n = entries.filter((e) => e.enabled).length;
      return {
        value: plural(n, "programa", "programas"),
        detail: "se abren al iniciar Windows",
        ok: n < LIMITS.startupEnabled,
      };
    },
  },
  {
    id: "updates",
    title: "Actualizaciones",
    module: "software-updater",
    load: async () => {
      const n = (await getUpdates()).length;
      return { value: plural(n, "programa", "programas"), detail: "con una versión más reciente", ok: n === 0 };
    },
  },
  {
    id: "privacy",
    title: "Privacidad y anuncios",
    module: "performance-optimizer",
    load: async () => {
      const tweaks = await invoke<{ applied: boolean; admin: boolean }[]>("list_tweaks");
      const n = tweaks.filter((t) => !t.applied && available(t)).length;
      return { value: plural(n, "ajuste", "ajustes"), detail: "de privacidad sin aplicar", ok: n === 0 };
    },
  },
];

const template = `
  <div class="toolbar">
    <button data-el="rescan" class="btn">Volver a analizar</button>
  </div>
  <div class="health">
    <div data-el="system" class="stats"><p class="row-note">Leyendo datos del equipo…</p></div>
    <div data-el="cards" class="cards"></div>
  </div>`;

function formatUptime(secs: number): string {
  const days = Math.floor(secs / 86400);
  const hours = Math.floor((secs % 86400) / 3600);
  const minutes = Math.floor((secs % 3600) / 60);
  return days ? `${days} d ${hours} h` : hours ? `${hours} h ${minutes} min` : `${minutes} min`;
}

function meter(used: number, total: number): string {
  const pct = total ? Math.round((100 * used) / total) : 0;
  return `<div class="meter ${pct >= 90 ? "high" : ""}"><div style="width: ${pct}%"></div></div>`;
}

function render(root: HTMLElement) {
  root.insertAdjacentHTML("beforeend", template);
  const el = <T extends HTMLElement>(name: string) => root.querySelector(`[data-el="${name}"]`) as T;
  const system = el("system");
  const cardsEl = el("cards");

  async function loadSystem() {
    try {
      const info = await invoke<SystemInfo>("system_info");
      const disks = info.disks
        .map(
          (d) => `
            <div class="stat">
              <span class="stat-label">Disco ${escapeHtml(d.letter)}</span>
              <strong>${formatBytes(d.free)} libres</strong>
              ${meter(d.total - d.free, d.total)}
              <span class="row-note">de ${formatBytes(d.total)}</span>
            </div>`,
        )
        .join("");
      system.innerHTML = `
        <div class="stat stat-wide">
          <span class="stat-label">Sistema</span>
          <strong>${escapeHtml(info.windows)}</strong>
          <span class="row-note">Encendido desde hace ${formatUptime(info.uptimeSecs)}</span>
        </div>
        <div class="stat">
          <span class="stat-label">Memoria</span>
          <strong>${formatBytes(info.memoryUsed)} en uso</strong>
          ${meter(info.memoryUsed, info.memoryTotal)}
          <span class="row-note">de ${formatBytes(info.memoryTotal)}</span>
        </div>
        ${disks}`;
    } catch (e) {
      system.innerHTML = `<p class="row-warning">No se pudieron leer los datos del equipo: ${escapeHtml(String(e))}</p>`;
    }
  }

  function renderCard(card: Card, state: "loading" | "error" | Finding, error?: string) {
    const target = cardsEl.querySelector<HTMLElement>(`[data-card="${card.id}"]`)!;
    const body =
      state === "loading"
        ? '<span class="row-note">Analizando…</span>'
        : state === "error"
          ? `<span class="row-warning">${escapeHtml(error ?? "Error")}</span>`
          : `<strong class="card-value">${escapeHtml(state.value)}</strong>
             <span class="row-note">${escapeHtml(state.detail)}</span>`;
    const tone = typeof state === "object" ? (state.ok ? "ok" : "warn") : "";
    const mark = tone === "ok" ? "✓" : tone === "warn" ? "!" : "";
    target.className = `card ${tone}`;
    target.innerHTML = `
      <span class="card-mark" aria-hidden="true">${mark}</span>
      <div class="card-main">
        <span class="stat-label">${escapeHtml(card.title)}</span>
        ${body}
      </div>
      <button class="btn small" data-module="${card.module}">Revisar</button>`;
  }

  function analyze(refreshUpdates: boolean) {
    for (const card of cards) {
      renderCard(card, "loading");
      const load = card.id === "updates" && refreshUpdates ? () => getUpdates(true).then(() => card.load()) : card.load;
      load().then(
        (finding) => renderCard(card, finding),
        (e) => renderCard(card, "error", String(e)),
      );
    }
  }

  cardsEl.innerHTML = cards.map((c) => `<div data-card="${c.id}"></div>`).join("");
  cardsEl.addEventListener("click", (event) => {
    const button = (event.target as HTMLElement).closest<HTMLButtonElement>("[data-module]");
    if (button) navigate(button.dataset.module!);
  });
  el("rescan").addEventListener("click", () => {
    loadSystem();
    analyze(true);
  });
  loadSystem();
  analyze(false);
}

export const healthCheck: Module = {
  id: "health-check",
  label: "Health Check",
  description: "Resumen del estado de tu equipo. Aquí no se cambia nada.",
  // Corazón.
  icon: icon(`<path d="M12 20s-7.5-4.6-7.5-10.2A4.3 4.3 0 0 1 12 7.2a4.3 4.3 0 0 1 7.5 2.6C19.5 15.4 12 20 12 20z"/>`),
  render,
};
