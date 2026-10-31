# Spec Delta

## Purpose

Dar en una sola pantalla una visión del estado del equipo y llevar al usuario al módulo que puede mejorar cada aspecto, sin modificar nada.

## ADDED Requirements

### Requirement: Datos del equipo

El sistema SHALL mostrar la edición y versión de Windows, el tiempo desde el último arranque, la memoria total y en uso, y el espacio libre y total de cada disco fijo.

#### Scenario: Windows 11
- **WHEN** el número de compilación es 22000 o superior
- **THEN** se muestra "Windows 11" aunque el registro indique "Windows 10"

### Requirement: Apartados del resumen

El sistema SHALL mostrar los apartados espacio liberable, bloatware presente, programas de inicio activados, actualizaciones disponibles y ajustes de privacidad sin aplicar, cada uno con su valor, un estado (correcto o mejorable) y un botón "Revisar" que abre el módulo correspondiente. Cada apartado SHALL cargarse de forma independiente.

#### Scenario: Revisar el espacio liberable
- **WHEN** el usuario pulsa "Revisar" en "Espacio liberable"
- **THEN** se abre Custom Clean

#### Scenario: Un apartado falla
- **WHEN** winget no está disponible
- **THEN** el apartado de actualizaciones muestra el error y los demás se muestran con normalidad

### Requirement: Solo lectura

El módulo Health Check MUST NOT modificar el sistema: solo ejecuta los análisis de solo lectura de los demás módulos.

#### Scenario: Analizar
- **WHEN** el usuario abre Health Check o pulsa "Volver a analizar"
- **THEN** no se borra, desinstala ni cambia ningún ajuste
