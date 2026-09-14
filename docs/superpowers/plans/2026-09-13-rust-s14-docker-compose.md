# Rust S14 Docker Compose Operational Stack Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task with review checkpoints.

**Goal:** Crear un stack Docker Compose reproducible para `prism-run` con TLS, autenticación, healthcheck, cliente de smoke test y apagado ordenado.

**Architecture:** Una imagen multi-stage contiene el runner y un cliente Python mínimo. Compose ejecuta `server` permanentemente y `client` solo bajo perfil; Docker healthcheck y restart cubren la supervisión del contenedor, mientras S13 sigue siendo la opción host-side.

**Tech Stack:** Dockerfile multi-stage, Docker Compose, Rust release, Python 3 estándar, TLS PEM y JSONL B2.

**Spec:** `docs/superpowers/specs/2026-09-13-rust-s14-docker-compose-design.md`

## Global Constraints

- La meta de capacidad es 100.000 frames.
- No incluir secretos en la imagen, `.env.example`, estado ni logs.
- Certificado, clave y token deben montarse en solo lectura.
- El contenedor debe iniciar `prism-run` con TLS y autenticación obligatorios.
- El apagado por `SIGTERM` debe activar el sentinel y respetar el drain timeout.
- No añadir mTLS, broker, persistencia, Kubernetes ni despliegue remoto.

---

### Task 1: Contratos de configuración Compose

**Files:**
- Create: `tests/test_s14_compose_contract.py`
- Create: `docker-compose.yml`
- Create: `.env.example`

**Interfaces:**
- Produce variables `PRISM_BIND`, `PRISM_ROUTE`, `PRISM_WORKERS`, `PRISM_CONNECTION_QUEUE`, `PRISM_DRAIN_TIMEOUT_MS`, `PRISM_HOST_PORT`, `PRISM_TLS_CERT`, `PRISM_TLS_KEY`, `PRISM_AUTH_TOKEN`.
- Produce servicios `server` y `client`, con el segundo bajo perfil `client`.

- [ ] **Step 1: Write failing contract tests** asserting service names, profile, read-only secret mounts, required environment placeholders and server command flags.
- [ ] **Step 2: Run `rtk proxy pytest tests/test_s14_compose_contract.py -q` and confirm failure because Compose files do not exist.**
- [ ] **Step 3: Add the minimal Compose and `.env.example` files satisfying the contract.**
- [ ] **Step 4: Run the focused tests and `rtk proxy docker compose config` when Docker is available; otherwise record the daemon blocker.**
- [ ] **Step 5: Append the task consumption row to `docs/consumo.md`.**

### Task 2: Image and lifecycle entrypoint

**Files:**
- Create: `Dockerfile`
- Create: `docker/entrypoint.sh`
- Create: `docker/healthcheck.py`
- Create: `docker/client.py`
- Test: `tests/test_s14_compose_contract.py`

**Interfaces:**
- Entrypoint consumes the Compose variables and starts `prism-run`.
- Healthcheck exits 0 only after a valid TLS-authenticated B2 response.
- Client prints the server JSON response and exits nonzero on failure.

- [ ] **Step 1: Extend the contract tests for Dockerfile stages, entrypoint signal handling and health/client scripts.**
- [ ] **Step 2: Run the focused tests and confirm the new assertions fail.**
- [ ] **Step 3: Implement the multi-stage image, shell entrypoint, healthcheck and client using only standard Python/TLS APIs.**
- [ ] **Step 4: Run focused tests and shell/script syntax checks available on the host.**
- [ ] **Step 5: Append the task consumption row to `docs/consumo.md`.**

### Task 3: Operational documentation and manual smoke flow

**Files:**
- Create: `scripts/docker_tls_smoke.ps1`
- Modify: `docs/guia-servidor-supervisor-cliente-tls.md`
- Test: `tests/test_s14_compose_contract.py`

**Interfaces:**
- Smoke script consumes Compose, certificate, key and token paths from `.env`/parameters and checks `up`, TLS client, logs and `down`.

- [ ] **Step 1: Add contract assertions for documented Compose commands and expected markers.**
- [ ] **Step 2: Run focused tests and confirm failure for the missing script/documentation.**
- [ ] **Step 3: Add the PowerShell smoke flow and a Compose section to the TLS guide.**
- [ ] **Step 4: Run focused tests and `rtk git diff --check`.**
- [ ] **Step 5: Append the task consumption row to `docs/consumo.md`.**

### Task 4: Full verification and evidence

**Files:**
- Create: `docs/evidence/rust-s14-docker-compose-2026-09-13.md`
- Modify: `docs/consumo.md`

- [ ] **Step 1: Run the S14 contract suite.**
- [ ] **Step 2: Run `rtk proxy 'C:\Users\ilver\.cargo\bin\cargo.exe' test --workspace --release`.**
- [ ] **Step 3: Run the Python suite.**
- [ ] **Step 4: If Docker is available, build and run the TLS smoke test; otherwise record the exact daemon error and do not claim runtime validation.**
- [ ] **Step 5: Write evidence with commands, exit codes, limitations and the unchanged 100.000-frame target.**
- [ ] **Step 6: Append the task and plan consumption rows to `docs/consumo.md`.**
