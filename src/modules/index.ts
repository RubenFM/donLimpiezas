import type { Module } from "./module";
import { healthCheck } from "./health-check";
import { customClean } from "./custom-clean";
import { uninstaller } from "./uninstaller";
import { bloatwareRemover } from "./bloatware-remover";
import { diskAnalyzer } from "./disk-analyzer";
import { duplicateFinder } from "./duplicate-finder";
import { cloudDriveCleaner } from "./cloud-drive-cleaner";
import { performanceOptimizer } from "./performance-optimizer";
import { startupManager } from "./startup-manager";
import { softwareUpdater } from "./software-updater";
import { driverUpdater } from "./driver-updater";
import { history } from "./history";

/**
 * Módulos en el orden de la barra lateral.
 */
export const modules: Module[] = [
  healthCheck,
  customClean,
  uninstaller,
  bloatwareRemover,
  diskAnalyzer,
  duplicateFinder,
  cloudDriveCleaner,
  performanceOptimizer,
  startupManager,
  softwareUpdater,
  driverUpdater,
  history,
];
