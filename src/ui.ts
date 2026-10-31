// Utilidades de interfaz compartidas por todos los módulos.

export function formatBytes(n: number): string {
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

export function escapeHtml(s: string): string {
  return s.replace(/[&<>"']/g, (c) => `&#${c.charCodeAt(0)};`);
}

export function groupBy<T extends { group: string }>(items: T[]): Map<string, T[]> {
  const groups = new Map<string, T[]>();
  for (const item of items) {
    if (!groups.has(item.group)) groups.set(item.group, []);
    groups.get(item.group)!.push(item);
  }
  return groups;
}

export function showStatus(el: HTMLElement, html: string, kind: "ok" | "error" | "info" = "info") {
  el.hidden = false;
  el.className = `status ${kind}`;
  el.innerHTML = html;
}

export function setBusy(button: HTMLButtonElement, busy: boolean, label?: string) {
  if (busy) {
    button.dataset.label = button.textContent ?? "";
    button.textContent = label ?? "Trabajando…";
  } else if (button.dataset.label) {
    button.textContent = button.dataset.label;
  }
  button.disabled = busy;
  document.body.classList.toggle("busy", busy);
}

export function confirmDialog(title: string, body: string): Promise<boolean> {
  const dialog = document.querySelector<HTMLDialogElement>("#confirm")!;
  dialog.querySelector("#confirm-title")!.textContent = title;
  dialog.querySelector("#confirm-body")!.innerHTML = body;
  dialog.returnValue = "";
  dialog.showModal();
  return new Promise((resolve) => {
    dialog.addEventListener("close", () => resolve(dialog.returnValue === "ok"), { once: true });
  });
}
