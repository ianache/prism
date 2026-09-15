# Rust S25 CI Evidence Gate Enforcement

Fecha: 2026-09-15  
Workflow: `Production Readiness`  
Run: `34933223766`
Commit probado: `1df74af`

## Resultado remoto

El run terminó `success` y todos los jobs pasaron: `contracts`, `compose`, `rust`,
`docker-build` y `production-runtime`. El job productivo verificó readiness TLS
con usuario no-root `10001`, el gate TCP comparable de `100000` tramas y el
smoke HTTP TLS de `3` tramas.

El artefacto estable publicado es `s25-production-evidence`. La evidencia
descargada fue auditada localmente con `scripts/audit_evidence_bundle.py`:

```text
ok pass=3 fail=0 not_executed=1 frames=100000
```

El bundle combinado contiene:

- `tcp smoke`: PASS, `3/3`.
- `http smoke`: PASS, `3/3`.
- `tcp 100000`: PASS, `100000/100000`.
- `http 100000`: NO EJECUTADA, con motivo explícito; HTTP 100K permanece una
  medición adicional y no sustituye el gate TCP principal.

Los archivos de evidencia no contienen PEM, tokens, credenciales ni payloads.
Los pasos de build, auditoría y upload usan `if: always()` únicamente para
conservar diagnóstico; el auditor mantiene fallo fatal si el bundle es inválido.

## Corrección aplicada

El builder ahora acepta un directorio HTTP separado para combinar el smoke
HTTP real con la evidencia TCP sin confundir `result.json` de un smoke con una
medición HTTP de `100000`. El workflow pasa `artifacts/s21-http` mediante
`--http-input-dir`.
