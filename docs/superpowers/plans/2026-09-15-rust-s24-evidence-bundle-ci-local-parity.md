# Rust S24 Evidence Bundle & CI/Local Parity Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Producir y validar un paquete de evidencia común para GitHub Actions y WSL2, conservando exactamente `100000` como meta comparable.

**Architecture:** Se definirá un esquema JSON versionado y un validador independiente en Python. CI y el launcher local generarán el mismo contrato, mientras la documentación y los tests comprobarán estados, comparabilidad y sanitización sin duplicar el runtime.

**Tech Stack:** Python, JSON Schema-like validation propia sin dependencias nuevas, PowerShell, Bash, GitHub Actions, pytest y Cargo release.

**Spec:** `docs/superpowers/specs/2026-09-15-rust-s24-evidence-bundle-ci-local-parity-design.md`

## Global Constraints

- TCP sigue siendo la decisión principal y HTTP `100000` es solo medición adicional.
- La aceptación comparable exige exactamente `frames_sent=100000` y `frames_ok=100000`.
- Smoke usa `3` frames, batch `1` y `synthetic_cycle=false`.
- `NO EJECUTADA` requiere motivo y no equivale a aceptación.
- No incluir PEM, tokens, credenciales, payloads ni variables secretas.
- No instalar automáticamente WSL2, distros Linux ni Docker Desktop.
- Mantener `PRISM_ALLOW_CYCLE=false` y `PRISM_CONTAINER_USER=10001:10001`.

---

### Task 1: Contrato del bundle y validador

**Files:**
- Create: `tools/evidence_bundle.py`
- Create: `tests/test_s24_evidence_bundle_contract.py`

**Interfaces:**
- Consumes: diccionario JSON de evidencia con metadatos y casos.
- Produces: `validate_bundle(bundle) -> list[str]`, lista vacía cuando el bundle es válido.

- [ ] Escribir tests fallidos para versión de esquema, metadatos obligatorios y cuatro estados/casos.
- [ ] Exigir reglas exactas para smoke `3/3`, gate TCP `100000/100000`, motivo de `NO EJECUTADA` y diagnóstico de `FAIL`.
- [ ] Implementar `validate_bundle` sin dependencias nuevas y con mensajes deterministas.
- [ ] Rechazar PEM, tokens, credenciales, payloads y nombres de variables secretas en cualquier valor serializado.
- [ ] Ejecutar `rtk proxy python -m pytest -q tests/test_s24_evidence_bundle_contract.py`.
- [ ] Commit: `feat: add S24 evidence bundle validator`.

### Task 2: Generador y auditoría del paquete

**Files:**
- Create: `scripts/build_evidence_bundle.py`
- Create: `tests/test_s24_evidence_bundle_builder.py`
- Modify: `scripts/run_production_gate.ps1`

**Interfaces:**
- Consumes: directorio de evidencia, `--source ci|wsl2`, commit y casos JSON sanitizados.
- Produces: `evidence-bundle.json` validado, sin copiar payloads ni secretos.

- [ ] Escribir tests fallidos para generación determinista, `source`, commit, digest y referencias relativas.
- [ ] Implementar CLI con `--input-dir`, `--output`, `--source`, `--commit` y `--digest`.
- [ ] Hacer que el launcher invoque el generador al terminar, manteniendo `PRISM_PROTOCOL`, frames, batch y código de salida.
- [ ] Si un caso no corre, emitir `NO EJECUTADA` con motivo y sin inventar métricas.
- [ ] Validar el bundle generado antes de escribirlo atómicamente.
- [ ] Ejecutar tests enfocados y parseo PowerShell.
- [ ] Commit: `feat: generate sanitized evidence bundles`.

### Task 3: Integración CI y paridad

**Files:**
- Modify: `.github/workflows/production-readiness.yml`
- Create: `tests/test_s24_ci_parity_contract.py`
- Modify: `docs/guia-servidor-supervisor-cliente-tls.md`

**Interfaces:**
- Consumes: `scripts/build_evidence_bundle.py` y artifacts S20/S21 existentes.
- Produces: artifact CI `s24-evidence-bundle` y comandos reproducibles para WSL2.

- [ ] Escribir tests fallidos para invocación CI con source `ci`, TCP `100000`, smoke y carga del artifact.
- [ ] Integrar generación/validación después del gate productivo sin debilitar pasos existentes.
- [ ] Mantener HTTP smoke como caso adicional y TCP como decisión comparable.
- [ ] Documentar la comparación CI/WSL2 por commit, digest, protocolo, frames y estado.
- [ ] Ejecutar contratos del workflow y validar ambos archivos Compose.
- [ ] Commit: `ci: publish S24 evidence bundle`.

### Task 4: Fixtures y auditoría de regresión

**Files:**
- Create: `tests/fixtures/s24/evidence-bundle-ci.json`
- Create: `tests/fixtures/s24/evidence-bundle-not-executed.json`
- Create: `tests/test_s24_evidence_audit.py`
- Create: `scripts/audit_evidence_bundle.py`

**Interfaces:**
- Consumes: bundles CI, WSL2 y `NO EJECUTADA`.
- Produces: salida `ok ...` o diagnóstico de incumplimiento, sin exponer valores sensibles.

- [ ] Escribir fixtures válidos para PASS comparable y NO EJECUTADA ambiental.
- [ ] Escribir fixtures inválidos para frames parciales, digest ausente, motivo ausente y secretos.
- [ ] Implementar auditoría que reporte conteos por estado y preserve `100000`.
- [ ] Ejecutar auditoría sobre los fixtures y escaneo dirigido de secretos.
- [ ] Commit: `test: audit S24 evidence parity`.

### Task 5: Verificación completa y cierre

**Files:**
- Modify: `docs/evidence/rust-s24-evidence-bundle-ci-local-parity-2026-09-15.md`
- Modify: `docs/consumo.md`
- Modify: `docs/superpowers/plans/2026-09-15-rust-s24-evidence-bundle-ci-local-parity.md`

- [ ] Ejecutar `rtk proxy python -m pytest -q` y registrar el conteo exacto.
- [ ] Ejecutar `rtk proxy 'C:\Users\ilver\.cargo\bin\cargo.exe' test --workspace --release`.
- [ ] Ejecutar `docker compose config` base/producción, parseo PowerShell y `git diff --check`.
- [ ] Auditar bundles válidos, inválidos y `NO EJECUTADA`; confirmar ausencia de secretos.
- [ ] Crear evidencia S24 con diferencia explícita entre CI PASS y WSL2 NO EJECUTADA si aplica.
- [ ] Actualizar `docs/consumo.md` con una fila por tarea y una fila del plan.
- [ ] Marcar el plan como completado solo después de observar las verificaciones.
- [ ] Commit: `docs: finalize S24 evidence bundle parity`.

## Completion Gate

S24 se completa cuando CI y WSL2 pueden producir el mismo bundle, el validador
rechaza evidencia incompleta o sensible, TCP `100000/100000` conserva su papel
de decisión, `NO EJECUTADA` sigue siendo explícito y todas las suites pasan.

## Cierre de ejecución

- [x] Tarea 1: contrato y validador.
- [x] Tarea 2: generador y bundle WSL2.
- [x] Tarea 3: integración CI y paridad.
- [x] Tarea 4: fixtures y auditoría.
- [x] Tarea 5: verificación y cierre auditable.

Resultado observado: Python `130 passed, 21 subtests passed`, Cargo release
PASS y validaciones Compose/PowerShell/seguridad PASS. La ejecución del
workflow CI queda pendiente de confirmación remota; el host WSL2 sigue sin
distro Linux de usuario y por tanto conserva estado `NO EJECUTADA`.

## Cierre de ejecución

- [x] Tarea 1: contrato y validador.
- [x] Tarea 2: generador y bundle WSL2.
- [x] Tarea 3: integración CI y paridad.
- [x] Tarea 4: fixtures y auditoría.
- [x] Tarea 5: verificación y cierre auditable.
