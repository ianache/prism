# Rust S25 CI Evidence Gate Enforcement Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Enforcear en GitHub Actions la validación y auditoría del bundle S24 antes de publicar evidencia productiva.

**Architecture:** El workflow existente conservará sus gates de runtime y añadirá una fase explícita de validación/auditoría. Los scripts S24 serán la única fuente de reglas; S25 agregará contratos del workflow, fixtures de CI inválidos y evidencia del run remoto.

**Tech Stack:** GitHub Actions, Python, JSON, pytest, Docker Compose y Cargo release.

**Spec:** `docs/superpowers/specs/2026-09-15-rust-s25-ci-evidence-gate-enforcement-design.md`

## Global Constraints

- TCP es la decisión principal y exige exactamente `frames_sent=100000` y `frames_ok=100000`.
- HTTP `100000` es medición adicional; HTTP smoke usa `3` frames y batch `1`.
- `NO EJECUTADA` requiere motivo y nunca equivale a PASS.
- No incluir PEM, tokens, credenciales, payloads ni valores secretos.
- Mantener `PRISM_ALLOW_CYCLE=false`, `PRISM_CONTAINER_USER=10001:10001` y `100000`.
- Publicar artifacts sanitizados con nombre estable `s25-production-evidence`.

---

### Task 1: Contrato de enforcement del workflow

**Files:**
- Create: `tests/test_s25_ci_evidence_gate_contract.py`
- Modify: `.github/workflows/production-readiness.yml`

**Interfaces:**
- Consumes: builder y auditor S24.
- Produces: pasos CI explícitos de build, validate, audit y upload.

- [ ] Escribir tests fallidos para exigir comandos builder/auditor, artifact estable y ejecución `if: always()`.
- [ ] Exigir que el workflow conserve gate TCP `100000/64` y HTTP smoke separado.
- [ ] Añadir pasos `Validate S24 evidence bundle` y `Audit S24 evidence bundle`.
- [ ] Publicar `s25-production-evidence` con el bundle y resultados sanitizados.
- [ ] Ejecutar el contrato del workflow y `git diff --check`.
- [ ] Commit: `ci: enforce S24 evidence gate`.

### Task 2: Fixtures de aceptación y rechazo CI

**Files:**
- Create: `tests/fixtures/s25/valid-bundle.json`
- Create: `tests/fixtures/s25/invalid-partial-gate.json`
- Create: `tests/fixtures/s25/invalid-secret.json`
- Create: `tests/test_s25_ci_evidence_validation.py`

- [ ] Escribir tests fallidos para aceptar el fixture válido y rechazar frames parciales.
- [ ] Rechazar secretos y payloads mediante el mismo validador S24 usado por CI.
- [ ] Comprobar que `NO EJECUTADA` con motivo es válido pero no es aceptación.
- [ ] Ejecutar tests enfocados y los scripts de auditoría sobre cada fixture.
- [ ] Commit: `test: add S25 CI evidence fixtures`.

### Task 3: Falla explícita y diagnóstico de CI

**Files:**
- Modify: `.github/workflows/production-readiness.yml`
- Create: `tests/test_s25_ci_failure_contract.py`
- Modify: `docs/guia-servidor-supervisor-cliente-tls.md`

- [ ] Escribir contrato para que un error del validator/auditor produzca exit code no cero.
- [ ] Mantener `if: always()` solo para conservar evidencia, no para ocultar el fallo.
- [ ] Documentar cómo inspeccionar el artifact y diferenciar FAIL de NO EJECUTADA.
- [ ] Validar el workflow mediante contratos estáticos y parseo YAML disponible.
- [ ] Commit: `docs: document CI evidence enforcement`.

### Task 4: Ejecución remota y evidencia S25

**Files:**
- Create: `docs/evidence/rust-s25-ci-evidence-gate-enforcement-2026-09-15.md`
- Modify: `docs/consumo.md`

- [x] Ejecutar el workflow en GitHub Actions mediante push/dispatch autorizado.
- [x] Registrar run, jobs, artifact, TCP `100000/100000`, smoke, lifecycle y sanitización.
- [x] Si CI falla, conservar diagnóstico y no declarar S25 completado.
- [x] Auditar el artifact descargado sin exponer secretos.
- [x] Commit: `test: capture S25 CI evidence gate run`.

### Task 5: Verificación completa y cierre

**Files:**
- Modify: `docs/evidence/rust-s25-ci-evidence-gate-enforcement-2026-09-15.md`
- Modify: `docs/consumo.md`
- Modify: `docs/superpowers/plans/2026-09-15-rust-s25-ci-evidence-gate-enforcement.md`

- [x] Ejecutar `rtk proxy python -m pytest -q`.
- [x] Ejecutar `rtk proxy 'C:\Users\ilver\.cargo\bin\cargo.exe' test --workspace --release`.
- [x] Ejecutar ambos `docker compose config`, parseo PowerShell y `git diff --check`.
- [x] Confirmar ausencia de secretos y preservación exacta de `100000`.
- [x] Actualizar consumo con una fila por tarea y una fila del plan.
- [x] Marcar el plan completado solo con run CI verde y artifact auditado.
- [x] Commit: `docs: finalize S25 CI evidence gate enforcement`.

## Completion Gate

S25 se completa cuando CI rechaza bundles inválidos, acepta el bundle válido,
publica `s25-production-evidence`, demuestra el gate TCP `100000/100000` y la
evidencia remota queda auditada y sanitizada.
