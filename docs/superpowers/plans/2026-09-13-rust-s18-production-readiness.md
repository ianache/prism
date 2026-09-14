# S18 Production Readiness Gate Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Automatizar en Linux el gate CI/runtime del stack Docker TLS non-root con 100.000 frames reales.

**Architecture:** GitHub Actions ejecutará verificaciones deterministas y un job Docker separado generará secretos/corpus temporales, levantará server production y ejecutará el cliente stream. Los scripts producirán evidencia sanitizada sin incorporar secretos ni corpus al repositorio.

**Tech Stack:** GitHub Actions, Docker Compose, Rust/Cargo, Python 3, PowerShell solo para el flujo Windows existente.

**Spec:** `docs/superpowers/specs/2026-09-13-rust-s18-production-readiness-design.md`

## Global Constraints

- La meta de frames permanece en `100000`.
- El runtime production usa UID/GID `10001:10001`.
- `PRISM_ALLOW_CYCLE=false` en toda evidencia de aceptación.
- Secretos, claves privadas y corpus generado permanecen fuera de Git.
- No se reclama P0, mTLS, broker, persistencia ni Kubernetes.
- Cada tarea termina con prueba ejecutable y registro en `docs/consumo.md`.

### Task 1: CI contract and verification matrix

**Files:**
- Create: `.github/workflows/production-readiness.yml`
- Create: `scripts/ci_contract.py`
- Create: `tests/test_s18_ci_contract.py`
- Modify: `docs/guia-servidor-supervisor-cliente-tls.md`

**Interfaces:**
- `scripts/ci_contract.py` accepts repository root and validates required workflow commands and forbidden tracked secret patterns.
- The workflow exposes jobs `rust`, `contracts`, `compose`, and `docker-build`.

- [ ] Write failing assertions for workflow job names, release Cargo command, Compose production overlay, and secret exclusion.
- [ ] Run the focused contract and observe failure because the workflow is absent.
- [ ] Add the workflow and contract scanner with explicit command strings.
- [ ] Run `python scripts/ci_contract.py .` and the focused contract; expect PASS.
- [ ] Append the Task 1 row to `docs/consumo.md`.

### Task 2: Linux production secret and identity preflight

**Files:**
- Create: `scripts/docker_production_preflight.py`
- Create: `tests/test_s18_production_preflight.py`
- Modify: `docker-compose.production.yml`
- Modify: `.env.production.example`

**Interfaces:**
- `docker_production_preflight.py` creates temporary PEM/token paths, verifies their owner/mode expectations, and emits a sanitized JSON summary.
- The production overlay keeps service user `10001:10001` and does not alter the protocol.

- [ ] Add tests for effective UID/GID expectation, `PRISM_ALLOW_CYCLE=false`, and absence of secret values in output.
- [ ] Run focused tests and observe failure because the preflight is absent.
- [ ] Implement preflight validation and production environment defaults.
- [ ] Run it in a Linux-compatible container or CI job and verify sanitized output.
- [ ] Append the Task 2 row to `docs/consumo.md`.

### Task 3: Runtime smoke, 100K acceptance, and lifecycle recovery

**Files:**
- Create: `scripts/docker_production_readiness.sh`
- Create: `tests/test_s18_runtime_contract.py`
- Modify: `docker/telemetry_client.py`
- Modify: `docs/evidence/rust-s17-production-hardening-2026-09-13.md`

**Interfaces:**
- The shell script accepts `--frames`, `--batch-size`, and `--evidence-dir`, defaults to 100000/64, and exits nonzero on any failed phase.
- Evidence fields are `commit`, `image_digest`, `dataset_sha256`, `frames_sent`, `frames_ok`, `synthetic_cycle`, `effective_user`, and `exit_code`.

- [ ] Add contract tests for phase names, `PRISM_ALLOW_CYCLE=false`, 100000 default, sanitized evidence fields, and cleanup trap.
- [ ] Run the focused contract and observe failure because the script is absent.
- [ ] Implement smoke, real stream, SIGTERM drain, interrupted-client, reconnect, and cleanup phases.
- [ ] Run a reduced target smoke, then run the 100000 target in production overlay; verify all counts and lifecycle markers.
- [ ] Append the Task 3 row to `docs/consumo.md`.

### Task 4: Evidence packaging and documentation

**Files:**
- Create: `docs/evidence/rust-s18-production-readiness-YYYY-MM-DD.md`
- Modify: `docs/guia-servidor-supervisor-cliente-tls.md`
- Modify: `docs/consumo.md`

- [ ] Add a contract test requiring digest, identity, lifecycle, counts, duration, and sanitized-log sections.
- [ ] Run the contract and observe failure until the evidence template exists.
- [ ] Document local Windows and CI Linux commands, failure interpretation, cleanup, and rollback.
- [ ] Generate the dated evidence document from the accepted runtime output.
- [ ] Append the Task 4 row and the S18 plan-complete row to `docs/consumo.md`.

### Task 5: Full verification and publication

**Files:**
- Modify: `docs/consumo.md`

- [ ] Run `cargo test --workspace --release`.
- [ ] Run Python compilation/contracts and both Compose configurations.
- [ ] Run `git diff --check` and verify no secrets/corpus/`graphify-out/` are staged.
- [ ] Review the evidence against every S18 acceptance criterion.
- [ ] Commit the implementation and evidence, then publish `main` only after the merged tree is green.
