# Rust S25 CI Evidence Gate Enforcement — diseño

Fecha: 2026-09-15

## Objetivo

Hacer obligatorio en GitHub Actions el bundle de evidencia S24: CI debe
validarlo, auditarlo y fallar explícitamente ante evidencia incompleta,
incomparable o sensible antes de publicar el artifact.

## Diseño

El job `production-runtime` conservará sus fases actuales de preflight, smoke,
gate TCP `100000/64`, lifecycle y HTTP smoke adicional. Después de producir los
resultados, generará `evidence-bundle.json`, ejecutará el validador y luego la
auditoría S24. El artifact estable se llamará `s25-production-evidence` y se
publicará incluso cuando una fase falle, para permitir diagnóstico sanitizado.

El enforcement aplicará estas reglas: TCP comparable solo pasa con
`frames_sent=100000` y `frames_ok=100000`; smoke exige `3/3` y
`synthetic_cycle=false`; `FAIL` requiere diagnóstico; `NO EJECUTADA` requiere
motivo y no otorga aceptación; no se permiten PEM, tokens, credenciales,
payloads ni variables secretas.

## Fuera de alcance

- Nuevos protocolos o flujos HTTP.
- Cambiar TCP default o la meta `100000`.
- Instalar WSL2, Docker o runners locales.
- Sustituir la evidencia Linux ya obtenida en CI.

## Aceptación

El workflow debe fallar con un bundle inválido, pasar con fixtures válidos,
publicar el artifact `s25-production-evidence` sanitizado y dejar evidencia del
primer run verde posterior a S24.
