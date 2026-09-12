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

## Registro de consumo de Superpowers

Para cada plan y cada tarea ejecutados mediante Superpowers, mantén un registro en `docs/consumo.md`.

### Reglas de registro

- Registra una fila al finalizar cada plan y otra al finalizar cada tarea; no agrupes varias tareas en una sola fila.
- Incluye como mínimo: fecha, plan, identificador y nombre de la tarea, estado, hora de inicio, hora de fin, tiempo transcurrido, tokens de entrada, tokens de salida, tokens totales, fuente de la medición y observaciones.
- Usa el tiempo transcurrido real siempre que esté disponible. Si no puede medirse con precisión, marca el valor como `N/D` y explica el motivo en observaciones.
- Registra los tokens reales cuando la herramienta o el proveedor los informe. No inventes valores; usa `N/D` cuando no estén disponibles y conserva la fuente de medición.
- Mantén el archivo como un historial acumulativo: añade entradas nuevas al final y no sobrescribas registros anteriores.
- Si una tarea falla, se cancela o queda bloqueada, registra igualmente su consumo y estado.
- Actualiza el registro antes de dar por completado el plan o la tarea.

### Formato

Usa la tabla definida en `docs/consumo.md`. Los tiempos deben estar en segundos y los tokens como números enteros cuando sean conocidos.