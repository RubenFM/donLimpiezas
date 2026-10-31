import { invoke } from "@tauri-apps/api/core";
import { setBadge } from "./nav";

export type Update = { name: string; id: string; version: string; available: string };

let pending: Promise<Update[]> | null = null;

/**
 * Actualizaciones disponibles. El contador de la barra lateral, Software Updater
 * y Health Check comparten una sola ejecución de winget, que tarda varios segundos.
 */
export function getUpdates(refresh = false): Promise<Update[]> {
  if (refresh || !pending) {
    const request = invoke<Update[]>("list_updates");
    pending = request;
    request.then(
      (updates) => setBadge("software-updater", updates.length),
      // Si falla, la siguiente llamada lo vuelve a intentar.
      () => {
        if (pending === request) pending = null;
      },
    );
  }
  return pending;
}
