# Evidencia S15: Compose Telemetry Ingestion 100K

Fecha: 2026-09-13

## Implementación

- Se añadió el perfil Compose `telemetry` con el servicio `telemetry-client`.
- El cliente lee archivos binarios ordenados, convierte a `payload_hex` y envía JSONL B2 por TLS.
- Cada solicitud usa un `request_id` determinista y valida la respuesta `ok=true`.
- El cliente calcula `dataset_sha256` y exige alcanzar `PRISM_FRAME_TARGET`.
- El dataset, certificado y token se montan como solo lectura.
- Se añadió `scripts/docker_telemetry_smoke.ps1` para una prueba pequeña y repetible.

## Verificaciones

| Comando | Resultado |
|---|---|
| contrato estructural S15 mediante Python | `S15_CONTRACT_OK` |
| `docker compose --env-file .env.example --profile telemetry config` | PASS |
| `python -m compileall -q docker` | PASS |
| sintaxis PowerShell del smoke | `S15_POWERSHELL_SYNTAX_OK` |
| `git diff --check` | PASS |
| `cargo test --workspace --release` | PASS; código 0 |
| runtime `up` + ingestión + `down` | PENDIENTE; Docker Desktop no expone el daemon |

La meta configurada por defecto es 100.000 frames. La ejecución runtime requiere
un dataset con al menos esa cantidad de tramas válidas y certificados PEM reales.
