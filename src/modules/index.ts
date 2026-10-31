import type { Module } from "./module";
import { customClean } from "./custom-clean";
import { bloatwareRemover } from "./bloatware-remover";
import { startupManager } from "./startup-manager";

/**
 * Módulos implementados, en el orden de la barra lateral:
 * Health Check, Custom Clean, Uninstaller, Bloatware Remover, Duplicate Finder,
 * Cloud Drive Cleaner, Performance Optimizer, Startup Manager, Software Updater,
 * Driver Updater.
 */
export const modules: Module[] = [customClean, bloatwareRemover, startupManager];
