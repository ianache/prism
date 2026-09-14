# S17 Production Hardening & Real Telemetry Dataset

## Objetivo

Mantener la meta de decisión en 100.000 frames y sustituir el ciclo sintético
basado en archivos por un corpus secuencial reproducible de tramas B2 válidas.

## Diseño

- `scripts/generate_telemetry_stream.py` genera un JSONL externo con una trama
  válida por línea, identificador estable y digest SHA-256 del payload stream.
- `docker/telemetry_client.py` consume `PRISM_INPUT_STREAM` secuencialmente;
  solo usa el escaneo de archivos para smoke/desarrollo y conserva batching,
  orden, backpressure y validación de respuestas.
- Compose mantiene el perfil Windows local con secretos root-readable, y añade
  un perfil `production` non-root UID 10001 sin `user: 0:0`.
- La salida del cliente distingue corpus real (`synthetic_cycle=false`) de
  ciclo explícito (`true`) y reporta conteos y digest.

## Aceptación

1. Contratos S17 en rojo antes de la implementación y verdes después.
2. Compose valida perfiles local, telemetry y production.
3. Corpus externo de 100.000 payloads distintos, válidos y con digest estable.
4. Ejecución TLS con 100.000 respuestas B2 `ok`, sin `PRISM_ALLOW_CYCLE`.
5. Suite Rust release, compilación Python y `git diff --check` verdes.

No se reclama una calificación P0 ni rendimiento de almacenamiento más allá de
la evidencia medida en el host donde se ejecute.
