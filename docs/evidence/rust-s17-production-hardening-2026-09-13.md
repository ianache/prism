# S17 Production Hardening & Real Telemetry Dataset

S17 validó el flujo TLS Docker con un corpus secuencial real de 100.000 payloads
distintos. La meta de decisión permanece en 100.000 frames.

- Formato: `prism-telemetry-jsonl-v1`, 100.000 registros de 105 bytes.
- Digest SHA-256: `a0bed9fff13f913938f826e3cdaafd9d6de158fd3f609054b7b0969197ec630d`.
- Resultado: `frames_sent=100000`, `frames_ok=100000`, `synthetic_cycle=false`.
- Transporte: TLS + autenticación, una conexión, batching 64.
- Smoke adicional: 3 frames, `synthetic_cycle=false`, digest
  `9f11200afb958cd94c1f3270d3c5e843f41ac792572933cec43cb4ee28e22013`.

El corpus se mantuvo fuera de Git y se montó como un único archivo para evitar la
enumeración lenta de 100.000 bind mounts en Windows. La prueba demuestra validez,
identidad, orden y aceptación B2; no reclama una calificación P0 ni rendimiento de
almacenamiento independiente del host.

La ejecución directa del cliente se hizo en la red Compose porque las corridas
interactivas de `docker compose run` recreaban/eliminaban el servidor durante la
captura; el servicio server permaneció healthy en la corrida aceptada.
