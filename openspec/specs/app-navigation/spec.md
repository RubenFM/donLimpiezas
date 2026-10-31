# app-navigation Specification

## Purpose
Organizar la app en módulos accesibles desde una barra lateral fija, de forma que el usuario siempre sepa dónde está y vuelva al último módulo que usó.

## Requirements

### Requirement: Barra lateral de módulos

La app SHALL mostrar una barra lateral fija con una entrada por módulo implementado, cada una con icono y nombre en inglés, en el orden: Health Check, Custom Clean, Uninstaller, Bloatware Remover, Duplicate Finder, Cloud Drive Cleaner, Performance Optimizer, Startup Manager, Software Updater, Driver Updater. Los módulos no implementados MUST NOT aparecer.

#### Scenario: Módulos disponibles en esta versión
- **WHEN** se abre la app
- **THEN** la barra lateral muestra "Custom Clean" y "Bloatware Remover", en ese orden

### Requirement: Cambio de módulo

Al seleccionar una entrada, la app SHALL mostrar el contenido de ese módulo, marcar la entrada como activa y conservar el estado de los demás módulos (resultados de análisis, selecciones) mientras la app siga abierta.

#### Scenario: Volver a un módulo ya analizado
- **WHEN** el usuario analiza en Custom Clean, cambia a Bloatware Remover y vuelve a Custom Clean
- **THEN** sigue viendo los resultados del análisis anterior con su selección

#### Scenario: Navegación con teclado
- **WHEN** el foco está en la barra lateral y el usuario pulsa flecha abajo y después Intro
- **THEN** se abre el módulo siguiente

### Requirement: Recordar el último módulo

La app SHALL abrir al iniciarse el último módulo que el usuario tenía activo. Si no hay ninguno guardado o ese módulo ya no existe, SHALL abrir el primero de la barra lateral.

#### Scenario: Reapertura
- **WHEN** el usuario cierra la app con Bloatware Remover activo y la vuelve a abrir
- **THEN** se abre Bloatware Remover

#### Scenario: Primer inicio
- **WHEN** la app se abre por primera vez
- **THEN** se abre Custom Clean

### Requirement: Aviso de permisos común

El aviso de "sin permisos de administrador" SHALL mostrarse en la parte superior del área de contenido, visible en todos los módulos.

#### Scenario: Cambiar de módulo sin permisos
- **WHEN** la app se ejecuta sin administrador y el usuario cambia de módulo
- **THEN** el aviso con el botón "Reiniciar como administrador" sigue visible
