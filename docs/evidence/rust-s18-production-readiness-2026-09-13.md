# S18 Production Readiness Gate

## Estado

**PARCIAL — implementación y contratos preparados; ejecución Linux non-root
pendiente.**

El host de desarrollo es Windows. Docker Desktop está disponible, pero no hay
un runtime Bash/WSL funcional (`/bin/bash` no existe), por lo que no se reclama
la aceptación Linux de S18 ni se inventan métricas de esa fase.

## Preparado

- Workflow CI: `.github/workflows/production-readiness.yml`.
- Scanner de CI y secretos: `scripts/ci_contract.py`.
- Preflight non-root: `scripts/docker_production_preflight.py`.
- Runner Linux: `scripts/docker_production_readiness.sh`.
- Configuración production: UID/GID `10001:10001` y `PRISM_ALLOW_CYCLE=false`.
- Contratos S18 y compilación Python verificados localmente.

## Evidencia heredada de S17

S17 ya validó en Docker Desktop el stream TLS real:

- `frames_sent=100000`, `frames_ok=100000`.
- `synthetic_cycle=false`.
- Dataset SHA-256:
  `a0bed9fff13f913938f826e3cdaafd9d6de158fd3f609054b7b0969197ec630d`.
- No se reutiliza como prueba de UID/GID Linux.

## Gate pendiente

Para cerrar S18 en un runner Linux se debe ejecutar:

```bash
bash scripts/docker_production_readiness.sh \
  --frames 100000 \
  --batch-size 64 \
  --evidence-dir ./artifacts/s18
```

La evidencia final debe incluir digest, UID/GID efectivo, `frames_ok`, duración,
`STARTING → READY → DRAINING → STOPPED`, reconexión y logs sanitizados. No debe
contener tokens, claves privadas ni el corpus JSONL.
