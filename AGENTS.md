# Instrucciones para agentes

## Uso de graphify

Usa `graphify` cuando el trabajo requiera comprender un conjunto de archivos, descubrir relaciones entre documentos o código, analizar la arquitectura del proyecto o construir contexto persistente para futuras consultas.

### Activación

- Si el usuario escribe `/graphify`, invoca primero la skill `graphify` y sigue sus instrucciones completas.
- Si el usuario solicita explícitamente un grafo de conocimiento, análisis de relaciones, comunidades o un informe de corpus, usa `graphify` aunque no escriba `/graphify`.
- Antes de explorar manualmente muchos archivos, considera ejecutar `graphify` sobre la carpeta relevante.

### Flujo recomendado

1. Ejecuta `graphify` sobre la ruta indicada; si no se indica una ruta, usa la carpeta actual (`.`).
2. Conserva los artefactos generados por la herramienta: HTML interactivo, JSON del grafo y `GRAPH_REPORT.md`.
3. Usa la salida del grafo para orientar la exploración posterior y las decisiones de implementación.
4. Distingue siempre entre relaciones `EXTRACTED`, `INFERRED` y `AMBIGUOUS`; no presentes inferencias como hechos confirmados.
5. Para consultas sobre el resultado existente, usa `graphify query`, `graphify path` o `graphify explain` según corresponda.

### Reglas operativas

- No omitas la detección inicial de archivos ni la auditoría de `graphify`.
- Respeta las advertencias de tamaño del corpus y solicita una ruta más acotada si la herramienta lo requiere.
- Usa `--update` para refrescar un grafo existente cuando solo hayan cambiado o aparecido algunos archivos.
- Usa `--mode deep` únicamente cuando se necesiten relaciones semánticas más detalladas.
- No reemplaces la validación del código, las pruebas o la lectura de fuentes críticas por el grafo; `graphify` sirve para descubrir y organizar contexto.
- No expongas archivos sensibles omitidos por la herramienta; informa solo el conteo cuando corresponda.

La referencia detallada de la skill se encuentra en `graphify/SKILL.md` dentro de la instalación local de agentes.