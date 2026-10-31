import { invoke } from "@tauri-apps/api/core";
import { app } from "../app";
import { confirmDialog, escapeHtml, formatBytes, groupBy, setBusy, showStatus } from "../ui";
import { icon, type Module } from "./module";

type Driver = { device: string; class: string | null; provider: string | null; version: string | null; date: string | null };
type DriverUpdate = {
  id: string;
  title: string;
  class: string | null;
  manufacturer: string | null;
  date: string | null;
  size: number | null;
};
type InstallResult = { results: { id: string; title: string; ok: boolean; code: number }[]; reboot: boolean };

/** Fabricante con página oficial, deducido del proveedor del controlador. */
function vendorOf(provider: string | null): { key: string; name: string } | null {
  const p = (provider ?? "").toLowerCase();
  if (p.includes("nvidia")) return { key: "nvidia", name: "NVIDIA" };
  if (p.includes("amd") || p.includes("advanced micro")) return { key: "amd", name: "AMD" };
  if (p.includes("intel")) return { key: "intel", name: "Intel" };
  return null;
}

const template = `
  <div class="sections">
    <section>
      <h3 class="section-title">Actualizaciones de Windows Update</h3>
      <div class="toolbar">
        <button data-el="search" class="btn">Buscar actualizaciones</button>
        <label class="option"><input type="checkbox" data-el="restore" /> Crear punto de restauración</label>
        <button data-el="install" class="btn primary" disabled>Instalar seleccionadas</button>
      </div>
      <div data-el="updates" class="box"><p class="empty">Buscando en Windows Update…</p></div>
      <div data-el="status" class="status" hidden></div>
    </section>
    <section>
      <h3 class="section-title">Tarjeta gráfica</h3>
      <p class="row-note">Los fabricantes publican sus controladores de gráficos antes que Windows Update. Usa su herramienta oficial.</p>
      <div data-el="gpus" class="box"><p class="empty">Cargando…</p></div>
    </section>
    <section>
      <h3 class="section-title">Controladores instalados</h3>
      <div class="toolbar">
        <input data-el="filter" class="input" type="search" placeholder="Filtrar por dispositivo, clase o proveedor" />
        <span data-el="count" class="summary"></span>
      </div>
      <div data-el="drivers" class="box"><p class="empty">Cargando controladores…</p></div>
    </section>
  </div>`;

function render(root: HTMLElement) {
  root.insertAdjacentHTML("beforeend", template);
  const el = <T extends HTMLElement>(name: string) => root.querySelector(`[data-el="${name}"]`) as T;
  const updatesEl = el("updates");
  const status = el("status");
  const btnSearch = el<HTMLButtonElement>("search");
  const btnInstall = el<HTMLButtonElement>("install");
  const optRestore = el<HTMLInputElement>("restore");
  const filter = el<HTMLInputElement>("filter");

  let updates: DriverUpdate[] = [];
  let drivers: Driver[] = [];

  optRestore.checked = app.isAdmin;
  optRestore.disabled = !app.isAdmin;

  function selectedIds(): string[] {
    return [...updatesEl.querySelectorAll<HTMLInputElement>("input:checked")].map((c) => c.value);
  }

  function updateInstallButton() {
    btnInstall.disabled = !app.isAdmin || selectedIds().length === 0;
    btnInstall.title = app.isAdmin ? "" : "Instalar controladores requiere administrador";
  }

  function renderUpdates() {
    updatesEl.innerHTML = updates.length
      ? updates
          .map(
            (u) => `
              <label class="row">
                <input type="checkbox" value="${escapeHtml(u.id)}" checked ${app.isAdmin ? "" : "disabled"} />
                <div class="row-main">
                  <div class="row-title">${escapeHtml(u.title)}</div>
                  <div class="row-desc">${[u.manufacturer, u.class, u.date].filter(Boolean).map((d) => escapeHtml(d!)).join(" · ")}</div>
                </div>
                <div class="row-size"><strong>${u.size ? formatBytes(u.size) : "—"}</strong></div>
              </label>`,
          )
          .join("")
      : '<p class="empty">Windows Update no tiene controladores nuevos para este equipo. 🎉</p>';
    if (updates.length && !app.isAdmin) {
      updatesEl.insertAdjacentHTML("beforeend", '<p class="row-note list-note">Para instalarlos, reinicia la app como administrador.</p>');
    }
    updateInstallButton();
  }

  async function search() {
    setBusy(btnSearch, true, "Buscando…");
    btnInstall.disabled = true;
    updatesEl.innerHTML = '<p class="empty">Buscando en Windows Update…</p>';
    try {
      updates = await invoke<DriverUpdate[]>("search_driver_updates");
      renderUpdates();
    } catch (e) {
      updatesEl.innerHTML = "";
      showStatus(status, escapeHtml(String(e)), "error");
    } finally {
      setBusy(btnSearch, false);
      updateInstallButton();
    }
  }

  async function install() {
    const ids = selectedIds();
    const titles = updates.filter((u) => ids.includes(u.id)).map((u) => escapeHtml(u.title));
    const ok = await confirmDialog(
      "¿Instalar estos controladores?",
      `Windows Update descargará e instalará:<br>• ${titles.join("<br>• ")}<br><br>Puede tardar varios minutos.`,
    );
    if (!ok) return;

    setBusy(btnInstall, true, "Instalando…");
    btnSearch.disabled = true;
    const lines: string[] = [];
    try {
      if (optRestore.checked) {
        showStatus(status, "Creando punto de restauración…");
        try {
          lines.push(escapeHtml(await invoke<string>("create_restore_point", { purpose: "drivers" })));
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
      showStatus(status, "Descargando e instalando con Windows Update…");
      const result = await invoke<InstallResult>("install_driver_updates", { ids });
      const okCount = result.results.filter((r) => r.ok).length;
      lines.push(`${okCount} de ${result.results.length} controladores instalados.`);
      for (const r of result.results.filter((r) => !r.ok)) {
        lines.push(`✗ ${escapeHtml(r.title)} (código ${r.code})`);
      }
      if (result.reboot) lines.push("<strong>Reinicia el equipo para terminar la instalación.</strong>");
      showStatus(status, lines.join("<br>"), okCount === result.results.length ? "ok" : "error");
    } catch (e) {
      showStatus(status, escapeHtml(String(e)), "error");
    } finally {
      setBusy(btnInstall, false);
      btnSearch.disabled = false;
    }
    await search();
  }

  function renderGpus() {
    const gpus = drivers.filter((d) => d.class === "DISPLAY");
    el("gpus").innerHTML = gpus.length
      ? gpus
          .map((g) => {
            const vendor = vendorOf(g.provider);
            return `
              <div class="row">
                <div class="row-main">
                  <div class="row-title">${escapeHtml(g.device)}</div>
                  <div class="row-desc">${[g.provider, g.version && `versión ${g.version}`, g.date].filter(Boolean).map((d) => escapeHtml(d!)).join(" · ")}</div>
                </div>
                ${vendor ? `<button class="btn small" data-vendor="${vendor.key}">Web de ${vendor.name}</button>` : ""}
              </div>`;
          })
          .join("")
      : '<p class="empty">No se ha encontrado ninguna tarjeta gráfica.</p>';
  }

  function renderDrivers() {
    const q = filter.value.trim().toLowerCase();
    const visible = drivers.filter(
      (d) => !q || [d.device, d.class, d.provider].some((v) => v?.toLowerCase().includes(q)),
    );
    el("count").textContent = `${visible.length} de ${drivers.length} controladores`;
    el("drivers").innerHTML = visible.length
      ? [...groupBy(visible.map((d) => ({ ...d, group: d.class ?? "Otros" })))]
          .map(
            ([group, items]) =>
              `<h3 class="group">${escapeHtml(group)}</h3>` +
              items
                .map(
                  (d) => `
                    <div class="row">
                      <div class="row-main">
                        <div class="row-title">${escapeHtml(d.device)}</div>
                        <div class="row-desc">${escapeHtml(d.provider ?? "")}</div>
                      </div>
                      <div class="row-size"><strong>${escapeHtml(d.version ?? "—")}</strong><span>${escapeHtml(d.date ?? "")}</span></div>
                    </div>`,
                )
                .join(""),
          )
          .join("")
      : '<p class="empty">Ningún controlador coincide con el filtro.</p>';
  }

  updatesEl.addEventListener("change", updateInstallButton);
  el("gpus").addEventListener("click", (event) => {
    const button = (event.target as HTMLElement).closest<HTMLButtonElement>("[data-vendor]");
    if (button) invoke("open_vendor_page", { vendor: button.dataset.vendor }).catch((e) => showStatus(status, escapeHtml(String(e)), "error"));
  });
  filter.addEventListener("input", renderDrivers);
  btnSearch.addEventListener("click", search);
  btnInstall.addEventListener("click", install);

  invoke<Driver[]>("list_drivers").then(
    (list) => {
      drivers = list;
      renderGpus();
      renderDrivers();
    },
    (e) => {
      el("gpus").innerHTML = el("drivers").innerHTML = `<p class="row-warning list-note">${escapeHtml(String(e))}</p>`;
    },
  );
  search();
}

export const driverUpdater: Module = {
  id: "driver-updater",
  label: "Driver Updater",
  description: "Controladores al día solo con Windows Update y las webs oficiales de los fabricantes.",
  // Chip.
  icon: icon(
    `<rect x="7" y="7" width="10" height="10" rx="1.5"/>
     <path d="M10 4v3M14 4v3M10 17v3M14 17v3M4 10h3M4 14h3M17 10h3M17 14h3"/>`,
  ),
  render,
};
