# Tasks

## 1. Backend

- [ ] 1.1 Añadir `default_selected`, `warning` y `processes` a `Category` y `defaultSelected`, `warning` y `running` a `CategoryScan`; verificar que `cargo build` compila sin warnings
- [ ] 1.2 Implementar la lista de procesos con ToolHelp (feature `Win32_System_Diagnostics_ToolHelp`) y verificar con un test que el propio proceso de test aparece
- [ ] 1.3 Añadir el destino `Files`, extraer `chromium_profiles` y crear las categorías de la tabla del diseño; verificar con `cargo test` que `categories()` no repite ids y que solo Privacidad tiene `default_selected = false`

## 2. Interfaz

- [ ] 2.1 En `custom-clean.ts`, usar `defaultSelected` para la selección inicial y mostrar `warning` y la etiqueta "Abierto"; verificar que `npx tsc --noEmit` y `npx vite build` pasan y revisar una captura con datos simulados

## 3. Integración y documentación

- [ ] 3.1 Ejecutar un análisis real (test `#[ignore]`) y comprobar que aparecen las categorías nuevas con tamaños plausibles y que las de un navegador abierto marcan `running`
- [ ] 3.2 Actualizar la tabla del `## Purpose` de `openspec/specs/system-cleanup/spec.md` y la sección de funciones del `README.md`; verificar que coinciden con `categories()`

## Workflow follow-up

- Probar en la app real y archivar con `/opsx:archive`.
