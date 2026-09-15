# S24 Evidence Bundle & CI/Local Parity — evidencia

Fecha: 2026-09-15  
Rama: `work/s24-evidence-bundle-ci-local-parity`

## Resultado

Se implementó un contrato `s24.v1`, un generador sanitizado y una auditoría
común para evidencia CI y WSL2. El validador conserva TCP como decisión
principal y exige exactamente `100000/100000`; HTTP `100000` queda como medición
adicional.

## Paquetes de referencia

- Fixture CI PASS: `tests/fixtures/s24/evidence-bundle-ci.json`.
- Fixture WSL2 ambiental: `tests/fixtures/s24/evidence-bundle-not-executed.json`.
- Auditoría: `python scripts/audit_evidence_bundle.py <bundle>`.
- Generación: `python scripts/build_evidence_bundle.py --input-dir <dir> --output <bundle> --source ci|wsl2 --commit <sha> --digest <digest>`.

## Estados

Los fixtures y contratos verifican `PASS`, `FAIL` y `NO EJECUTADA`. La última
categoría requiere motivo y no constituye aceptación. En el host de desarrollo
S23, la ejecución WSL2 real sigue `NO EJECUTADA` porque no existe una distro
Linux de usuario; GitHub Actions continúa siendo la evidencia Linux operativa.

## Verificación

- Suite Python completa: `130 passed, 21 subtests passed`.
- Suite Cargo workspace release: PASS.
- Contratos S24 y auditoría: PASS.
- El bundle CI de referencia audita `pass=4`, `fail=0`, `not_executed=0`.
- El bundle ambiental audita `pass=0`, `fail=0`, `not_executed=4`.
- Se rechaza evidencia sensible, payloads, tokens, credenciales y PEM.
- El workflow publica `s24-evidence-bundle`; la ejecución del workflow debe
  confirmar el artifact en GitHub Actions.
- La meta comparable permanece exactamente `100000` frames.
