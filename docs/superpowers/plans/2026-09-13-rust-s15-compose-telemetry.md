# Rust S15 Compose Telemetry Ingestion 100K Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task with review checkpoints.

**Goal:** Añadir ingestión determinista de tramas binarias desde Compose hacia el servidor B2 TLS.

**Architecture:** El nuevo servicio `telemetry-client` se ejecuta bajo perfil Compose, lee archivos binarios montados como solo lectura y usa TLS + token para enviar sobres JSONL al servicio `server`. El cliente conserva orden, cuenta respuestas y emite un resumen verificable.

**Tech Stack:** Docker Compose, Python 3 estándar, socket TLS, JSONL, SHA-256 y fixtures binarios existentes.

**Spec:** `docs/superpowers/specs/2026-09-13-rust-s15-compose-telemetry-design.md`

## Global Constraints

- La meta de capacidad es 100.000 frames.
- No incluir secretos en imágenes, datasets ni logs.
- Dataset, certificados y token se montan como solo lectura.
- El transporte es TLS + JSONL B2 autenticado.
- El cliente falla si no alcanza el objetivo o recibe respuestas inválidas.
- No añadir broker, persistencia ni reintentos semánticos.

---

### Task 1: Contrato del cliente de telemetría

**Files:**
- Modify: `tests/test_s14_compose_contract.py`
- Create: `tests/test_s15_telemetry_contract.py`
- Modify: `docker-compose.yml`
- Modify: `.env.example`

- [ ] Write failing tests for profile `telemetry`, variables de dataset/objetivo y script de cliente.
- [ ] Run the focused test and confirm failure because the service/script is missing.
- [ ] Add the `telemetry-client` service, read-only dataset mount and variables.
- [ ] Run focused contract checks and `docker compose --profile telemetry config`.
- [ ] Record consumption.

### Task 2: Cliente determinista de tramas

**Files:**
- Create: `docker/telemetry_client.py`
- Modify: `Dockerfile`
- Modify: `tests/test_s15_telemetry_contract.py`

- [ ] Test file ordering, frame target, request IDs and summary output.
- [ ] Confirm the tests fail before the client exists.
- [ ] Implement standard-library TLS JSONL client with bounded target and SHA-256 summary.
- [ ] Run contract tests and `python -m compileall -q docker`.
- [ ] Record consumption.

### Task 3: Smoke script and documentation

**Files:**
- Create: `scripts/docker_telemetry_smoke.ps1`
- Modify: `docs/guia-servidor-supervisor-cliente-tls.md`
- Modify: `tests/test_s15_telemetry_contract.py`

- [ ] Add assertions for documented telemetry commands and expected marker.
- [ ] Confirm the assertions fail before the script/docs section exists.
- [ ] Add a small smoke command and a 100K command using `--profile telemetry`.
- [ ] Run syntax, diff and config checks.
- [ ] Record consumption.

### Task 4: Verification and evidence

**Files:**
- Create: `docs/evidence/rust-s15-compose-telemetry-2026-09-13.md`
- Modify: `docs/consumo.md`

- [ ] Run structural S15 checks and full Rust release suite.
- [ ] Run Docker Compose config with server, client and telemetry profiles.
- [ ] Run Docker runtime smoke when the daemon is available; otherwise record the blocker.
- [ ] Document exact counts, target and limitations.
- [ ] Record task and plan consumption rows.
