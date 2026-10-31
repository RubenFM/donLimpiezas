// Estado global mínimo, inicializado antes de renderizar ningún módulo.

import { invoke } from "@tauri-apps/api/core";

export const app = {
  isAdmin: false,
};

export async function initApp() {
  app.isAdmin = await invoke<boolean>("is_admin");
}
