import type { Module } from "./module";
import { customClean } from "./custom-clean";
import { uninstaller } from "./uninstaller";
import { bloatwareRemover } from "./bloatware-remover";
import { duplicateFinder } from "./duplicate-finder";
import { performanceOptimizer } from "./performance-optimizer";
import { startupManager } from "./startup-manager";

/**
 * Módulos implementados, en el orden de la barra lateral:
 * Health Check, Custom Clean, Uninstaller, Bloatware Remover, Duplicate Finder,
 * Cloud Drive Cleaner, Performance Optimizer, Startup Manager, Software Updater,
 * Driver Updater.
 */
export const modules: Module[] = [
  customClean,
  uninstaller,
  bloatwareRemover,
  duplicateFinder,
  performanceOptimizer,
  startupManager,
];
