# Hoja de ruta

## Hecho

| Módulo | Especificación | Cambios archivados |
|--------|----------------|--------------------|
| Navegación lateral | `app-navigation` | `add-sidebar-navigation`, `add-software-updater`, `add-health-check` |
| Health Check | `health-check` | `add-health-check` |
| Custom Clean | `system-cleanup` | base, `extend-custom-clean`, `add-component-store-cleanup` |
| Uninstaller | `program-uninstall` | `add-uninstaller` |
| Bloatware Remover | `bloatware-removal` | base |
| Disk Analyzer | `disk-analysis` | `add-disk-analyzer` |
| Duplicate Finder | `duplicate-files` | `add-duplicate-finder` |
| Cloud Drive Cleaner | `cloud-storage-cleanup` | `add-cloud-drive-cleaner` |
| Performance Optimizer | `system-tweaks` | `add-performance-optimizer` |
| Startup Manager | `startup-programs` | `add-startup-manager`, `extend-startup-manager` |
| Software Updater | `software-updates` | `add-software-updater` |
| Driver Updater | `driver-updates` | `add-driver-updater` |
| History | `activity-history` | `add-activity-history` |
| Permisos | `privileges` | base |

Los cambios están en `openspec/changes/archive/`.

## Ideas pendientes

- Limpieza programada (Programador de tareas): necesita un ejecutable instalado; se retoma cuando haya despliegue.
- Ajustes de la app: exclusiones en Custom Clean, umbrales de Health Check, idioma.
- Versión en inglés de los textos.

Flujo por cambio: `/opsx:propose` → `/opsx:apply` → probar en la app → `/opsx:archive`.
