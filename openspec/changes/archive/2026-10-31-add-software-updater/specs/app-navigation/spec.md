# Spec Delta

## ADDED Requirements

### Requirement: Contador de avisos por módulo

Un módulo SHALL poder comprobar datos en segundo plano al arrancar la app, sin abrirse, y mostrar en su entrada de la barra lateral un contador de avisos. El contador MUST desaparecer cuando no hay avisos y MUST seguir visible en la barra contraída.

#### Scenario: Actualizaciones pendientes al arrancar
- **WHEN** se abre la app y hay 37 actualizaciones disponibles
- **THEN** la entrada "Software Updater" muestra "37" sin necesidad de abrir el módulo
