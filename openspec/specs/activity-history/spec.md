# activity-history Specification

## Purpose
Ver en un solo sitio todo lo que la app ha cambiado en el equipo y deshacerlo cuando sea posible.

## Requirements

### Requirement: Historial unificado

El sistema SHALL mostrar las acciones registradas en los logs de todos los módulos (Bloatware Remover, Startup Manager, Performance Optimizer, Uninstaller, Duplicate Finder, Cloud Drive Cleaner, Software Updater y Driver Updater), de la más reciente a la más antigua, agrupadas por día, con módulo, descripción, hora y si tuvo éxito. SHALL permitir filtrar por módulo. Las líneas que no se puedan interpretar MUST ignorarse sin impedir mostrar las demás.

#### Scenario: Ajustes aplicados
- **WHEN** el usuario aplicó "game_dvr" a las 15:15
- **THEN** el historial muestra en el día de hoy "Ajuste aplicado: Grabación en segundo plano de juegos" a las 15:15

#### Scenario: Línea dañada
- **WHEN** un log contiene una línea sin fecha
- **THEN** esa línea se ignora y el resto se muestra

### Requirement: Deshacer cuando sigue siendo válido

El sistema SHALL ofrecer deshacer una acción solo si el estado actual sigue siendo el que dejó esa acción. Al deshacer SHALL recalcular la acción a partir del log (no de datos enviados por la interfaz) y registrarla como una acción nueva.

#### Scenario: Programa de inicio desactivado
- **WHEN** el usuario desactivó Spotify del inicio y sigue desactivado
- **THEN** la entrada ofrece "Volver a activar" y, al pulsarlo, Spotify vuelve a iniciarse con Windows

#### Scenario: Ya cambiado después
- **WHEN** el usuario desactivó Spotify y luego lo reactivó
- **THEN** la entrada antigua no ofrece deshacer

#### Scenario: Ajuste aplicado
- **WHEN** un ajuste aplicado tiene su valor original guardado
- **THEN** la entrada ofrece "Restaurar"

#### Scenario: Archivo liberado de la nube
- **WHEN** el usuario dejó un vídeo de OneDrive solo en la nube y sigue así
- **THEN** la entrada ofrece "Volver a descargar", que lo marca como "Mantener siempre en este dispositivo"

### Requirement: Acciones de recuperación sin deshacer directo

Para acciones que la app no puede revertir por sí misma, el sistema SHALL ofrecer la vía de recuperación: buscar en Microsoft Store una app quitada del catálogo de Bloatware Remover y abrir la papelera para los duplicados eliminados. Las desinstalaciones y actualizaciones MUST mostrarse sin acción.

#### Scenario: App quitada
- **WHEN** el usuario quitó "Noticias"
- **THEN** la entrada ofrece "Buscar en Microsoft Store", que abre la Store buscando "Noticias"
