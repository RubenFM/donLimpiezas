# Spec Delta

## ADDED Requirements

### Requirement: Limpieza del almacén de componentes

El sistema SHALL ofrecer la categoría "Componentes antiguos de Windows Update", que requiere administrador, aparece desmarcada por defecto con el aviso de que tarda varios minutos y ejecuta la limpieza oficial de componentes de Windows sin `/ResetBase`. Su tamaño SHALL mostrarse como desconocido al analizar y, al limpiar, el espacio liberado SHALL calcularse como la diferencia del espacio libre del disco del sistema.

#### Scenario: Analizar
- **WHEN** el usuario analiza
- **THEN** la categoría aparece con tamaño "—" y desmarcada

#### Scenario: Limpiar
- **WHEN** el usuario la selecciona y limpia como administrador
- **THEN** se ejecuta la limpieza de componentes y se informa del espacio liberado en el disco del sistema

#### Scenario: Fallo de DISM
- **WHEN** DISM termina con error
- **THEN** la categoría se cuenta como fallida y el resto de categorías se limpia con normalidad
