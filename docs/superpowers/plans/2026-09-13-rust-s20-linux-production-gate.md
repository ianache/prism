# Rust S20 Linux Production Gate Closure Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Cerrar en GitHub Actions el gate Linux real del stack Docker Compose con TLS, non-root, lifecycle/reconnect y exactamente 100000 frames, dejando evidencia auditable y promoviendo solo un resultado verde.

**Architecture:** El workflow de readiness será la puerta única. Un disparo manual y los eventos existentes ejecutarán jobs encadenados de Rust, contratos, Compose, build y runtime; el runtime reutilizará el script S18 y publicará evidencia sanitizada S20. El runner Linux local queda fuera de alcance y permanece como backlog.

**Tech Stack:** GitHub Actions Ubuntu, Docker Compose, Bash, Python/pytest, Rust/Cargo release, TLS ephemeral secrets.

**Spec:** `docs/superpowers/specs/2026-09-13-rust-s20-linux-production-gate-design.md`

## Global Constraints

- La meta de decisión es exactamente `100000` frames.
- El runtime debe conservar `PRISM_CONTAINER_USER=10001:10001` y `PRISM_ALLOW_CYCLE=false`.
- No se aceptan secretos PEM, tokens ni material sensible en Git ni artifacts.
- No se relajan aserciones para convertir un fallo del gate en éxito.
- El runner Linux local no se implementa en S20.
- Cada tarea termina con prueba verificable y commit independiente.

---

### Task 1: Actualizar workflow y contratos S20

**Files:**
- Modify: `.github/workflows/production-readiness.yml`
- Modify: `tests/test_s18_ci_contract.py`
- Test: `tests/test_s18_ci_contract.py`

**Interfaces:**
- Consumes: jobs actuales `rust`, `contracts`, `compose`, `docker-build` y `production-runtime`.
- Produces: workflow invocable mediante `workflow_dispatch`, artifact `s20-linux-production-gate` y ruta `artifacts/s20`.

- [ ] **Step 1: Write the failing contract assertions**

Añadir al contrato S18:

```python
assert "workflow_dispatch:" in workflow
assert "--evidence-dir ./artifacts/s20" in workflow
assert "name: s20-linux-production-gate" in workflow
assert "path: artifacts/s20/" in workflow
```

- [ ] **Step 2: Run the focused contract to verify it fails**

Run: `python -m pytest tests/test_s18_ci_contract.py -q`

Expected: FAIL porque el workflow aún no contiene `workflow_dispatch` ni las referencias S20.

- [ ] **Step 3: Implement the workflow contract**

En `on`, agregar:

```yaml
  workflow_dispatch:
```

En `production-runtime`, cambiar únicamente las referencias de evidencia a:

```yaml
run: bash scripts/docker_production_readiness.sh --frames 100000 --batch-size 64 --evidence-dir ./artifacts/s20
name: s20-linux-production-gate
path: artifacts/s20/
```

- [ ] **Step 4: Run the focused contract to verify it passes**

Run: `python -m pytest tests/test_s18_ci_contract.py -q`

Expected: PASS.

- [ ] **Step 5: Commit**

```bash
git add .github/workflows/production-readiness.yml tests/test_s18_ci_contract.py
git commit -m "ci: expose S20 production gate workflow"
```

### Task 2: Añadir contrato automatizado del runtime S20

**Files:**
- Create: `tests/test_s20_production_gate_contract.py`
- Test: `tests/test_s20_production_gate_contract.py`
- Read-only reference: `scripts/docker_production_readiness.sh`

**Interfaces:**
- Consumes: script Bash S18 existente y sus flags `--frames`, `--batch-size`, `--evidence-dir`.
- Produces: protección contra regresiones de defaults, fases y aserciones de aceptación.

- [ ] **Step 1: Write the failing contract test**

Crear:

```python
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]


def test_production_gate_keeps_fixed_100k_and_required_phases():
    text = (ROOT / "scripts" / "docker_production_readiness.sh").read_text(encoding="utf-8")
    assert "frames=100000" in text
    assert 'batch_size=64' in text
    for phase in ("phase=config", "phase=preflight", "phase=smoke", "phase=100k", "phase=lifecycle", "phase=reconnect", "phase=cleanup"):
        assert phase in text
    assert '"frames_sent": ' in text
    assert '"frames_ok": ' in text
    assert '"synthetic_cycle": false' in text


def test_production_gate_uses_non_root_and_disables_synthetic_cycle():
    text = (ROOT / "scripts" / "docker_production_readiness.sh").read_text(encoding="utf-8")
    assert "PRISM_CONTAINER_USER=10001:10001" in text
    assert "PRISM_ALLOW_CYCLE=false" in text
```

- [ ] **Step 2: Run the new test**

Run: `python -m pytest tests/test_s20_production_gate_contract.py -q`

Expected: PASS against the existing S18 runner; if a required assertion fails, update the runner without weakening the criterion.

- [ ] **Step 3: Run shell syntax validation**

Run: `bash -n scripts/docker_production_readiness.sh`

Expected: exit code 0 on Ubuntu CI. On Windows, record the known absence of Bash and do not interpret it as Linux runtime evidence.

- [ ] **Step 4: Commit**

```bash
git add tests/test_s20_production_gate_contract.py
git commit -m "test: lock S20 production gate criteria"
```

### Task 3: Ejecutar la matriz local previa al CI

**Files:**
- Modify: `docs/consumo.md`
- Test: existing Rust and Python suites

**Interfaces:**
- Consumes: workflow/runner contracts from Tasks 1–2.
- Produces: baseline local PASS y registro de consumo sin afirmar capacidad Linux no ejecutada.

- [ ] **Step 1: Run Python contracts and scanner**

Run:

```powershell
python -m pytest tests/test_s14_compose_contract.py tests/test_s15_telemetry_contract.py tests/test_s17_stream_contract.py tests/test_s18_ci_contract.py tests/test_s20_production_gate_contract.py
python scripts/ci_contract.py .
```

Expected: all selected tests pass and scanner prints `S18_CI_CONTRACT_OK`.

- [ ] **Step 2: Run Rust release suite**

Run: `cargo test --workspace --release`

Expected: exit code 0.

- [ ] **Step 3: Validate Compose and repository hygiene**

Run:

```powershell
docker compose config
docker compose -f docker-compose.yml -f docker-compose.production.yml config
git diff --check
```

Expected: both Compose configurations resolve and no whitespace errors exist.

- [ ] **Step 4: Record the local baseline**

Append one S20 task row to `docs/consumo.md` with status `COMPLETADO` for local verification and explicitly state that the Linux runtime is still pending CI.

- [ ] **Step 5: Commit**

```bash
git add docs/consumo.md
git commit -m "docs: record S20 local verification baseline"
```

### Task 4: Ejecutar y auditar el gate Ubuntu

**Files:**
- Create: `docs/evidence/rust-s20-linux-production-gate-2026-09-13.md`
- Modify: `docs/consumo.md`
- Read: GitHub Actions run artifacts from `s20-linux-production-gate`

**Interfaces:**
- Consumes: workflow and local baseline from Tasks 1–3.
- Produces: run ID, job results, sanitized artifact audit and explicit PASS/PARCIAL.

- [ ] **Step 1: Publish the S20 branch and trigger CI**

Run:

```bash
git push -u origin work/s20-linux-production-gate
gh workflow run production-readiness.yml --ref work/s20-linux-production-gate
```

Expected: a new workflow run is created for the S20 branch.

- [ ] **Step 2: Wait for and inspect the run**

Run: `gh run watch <run-id> --exit-status`

Expected: `rust`, `contracts`, `compose`, `docker-build` and `production-runtime` all pass. If a job fails, keep the run as diagnostic evidence, identify the exact failing assertion/log line, fix only that cause in a follow-up commit, and rerun the same gate.

- [ ] **Step 3: Download and audit the artifact**

Run:

```bash
gh run download <run-id> -n s20-linux-production-gate -D artifacts/s20-audit
grep -R -E 'BEGIN .*PRIVATE KEY|s18-ephemeral-token|auth-token|PRISM_AUTH_TOKEN=' artifacts/s20-audit
```

Expected: the grep returns no matches; `result.json`/`telemetry.log` show `frames_sent=100000`, `frames_ok=100000`, `synthetic_cycle=false`, and `preflight.json` shows the expected non-root contract.

- [ ] **Step 4: Write sanitized evidence**

Document commit, run URL/ID, job results, artifact file list, acceptance fields and any limitation in `docs/evidence/rust-s20-linux-production-gate-2026-09-13.md`. Never copy PEM contents, tokens, or sensitive absolute paths.

- [ ] **Step 5: Commit evidence and consumption**

```bash
git add docs/evidence/rust-s20-linux-production-gate-2026-09-13.md docs/consumo.md
git commit -m "docs: capture S20 Linux production gate evidence"
```

### Task 5: Integrar únicamente con gate verde

**Files:**
- Modify: `docs/consumo.md`
- Read: final branch diff and GitHub Actions result

**Interfaces:**
- Consumes: successful S20 run and sanitized evidence from Task 4.
- Produces: fast-forward merge to `main`, remote publication and completed plan record.

- [ ] **Step 1: Verify final branch state**

Run:

```bash
git status --short
git log --oneline --decorate -5
gh run view <run-id> --json conclusion, jobs,url
```

Expected: clean worktree and `conclusion: success` for the S20 run.

- [ ] **Step 2: Mark the plan complete only after success**

Append the S20 plan row and task 5 row to `docs/consumo.md` with `COMPLETADO`, including the actual run ID and commit hashes. If CI is not green, use `COMPLETADO PARCIAL` and do not merge.

- [ ] **Step 3: Merge and publish**

```bash
git switch main
git pull --ff-only origin main
git merge --ff-only work/s20-linux-production-gate
git push origin main
```

Expected: `main` contains the verified S20 commits and remote push succeeds.

- [ ] **Step 4: Confirm remote state**

Run: `git status --short`

Expected: clean `main`; preserve unrelated `graphify-out/` as untracked and do not add it.
