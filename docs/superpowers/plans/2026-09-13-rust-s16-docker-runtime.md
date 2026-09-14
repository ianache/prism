# Rust S16 Docker Runtime Validation & Telemetry Acceptance Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task with review checkpoints.

**Goal:** Cerrar la validación runtime del stack Compose TLS y aceptar 100.000 solicitudes B2 sintéticas reproducibles.

**Architecture:** Docker Desktop ejecuta `server` y `telemetry-client`; secretos Compose se montan de forma local, el cliente usa una conexión TLS ordenada con batching opcional y el entrypoint convierte TERM en sentinel de drain.

**Tech Stack:** Docker Desktop, Docker Compose, Rust release, Python TLS/JSONL, PowerShell.

**Spec:** `docs/superpowers/specs/2026-09-13-rust-s16-docker-runtime-design.md`

## Global Constraints

- Mantener la meta de 100.000 frames.
- No incluir secretos en imágenes ni Git.
- Marcar explícitamente las corridas `synthetic_cycle`.
- No reclamar un corpus distinto de 100.000 frames cuando se reutilicen fixtures.
- Mantener lifecycle `DRAINING → STOPPED`.

---

### Task 1: Preflight y secretos efímeros

- [x] Confirmar Docker Desktop y dataset disponible.
- [x] Generar PEM/token temporales fuera del repositorio.
- [x] Resolver Compose con secretos y perfiles.

### Task 2: Smoke y lifecycle

- [x] Construir imagen multi-stage.
- [x] Ejecutar smoke TLS de 4 frames.
- [x] Corregir permisos de secretos Windows mediante adaptación Compose root-only.
- [x] Corregir manejo de TERM en entrypoint.
- [x] Verificar `DRAINING` y `STOPPED` en logs.

### Task 3: Aceptación 100K

- [x] Detectar que 100.000 archivos en bind mount Windows ralentizan la enumeración.
- [x] Añadir ciclo sintético explícito y batching ordenado.
- [x] Ejecutar 100.000 solicitudes y validar conteos/digest.

### Task 4: Evidencia final

- [x] Registrar resultados, limitaciones y consumo.
- [x] Ejecutar verificaciones de formato, Compose, Python y Rust.
