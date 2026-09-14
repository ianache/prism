# Evidencia Rust S13 — Process Supervision and Release Hardening

Fecha: 2026-09-13. Alcance: wrapper PowerShell portable para operar el
runner TLS/auth local; la meta de 100.000 frames no cambia.

## Implementación

- `s13_supervisor.ps1` implementa `start`, `check` y `stop`.
- El estado persistido contiene únicamente PID, address, sentinel y stderr.
- `check` usa TLS y envía un sobre JSONL autenticado por la ruta B2.
- `stop` crea el sentinel y exige la observación de `STOPPED`.
- PID y nombre de proceso se validan antes de operar sobre estado existente.

## Verificación

- El wrapper se parsea y rechaza correctamente un estado inexistente/stale.
- Cargo workspace release, suite Python, formato y `git diff --check` pasan.
- La matriz TLS S9/S10 de S12 permanece verde.

No se incluyen instalación como servicio Windows, broker, persistencia,
Kubernetes, despliegue remoto, mTLS, rotación ni calificación P0.
