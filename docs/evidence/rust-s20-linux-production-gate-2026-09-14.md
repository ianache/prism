# Rust S20 — Linux Production Gate

Fecha: 2026-09-14  
Rama: `work/s20-linux-production-gate`  
Commit verificado: `a419be9`  
Run: `34809776551`

## Resultado

El workflow [Production Readiness run 34809776551](https://github.com/ianache/prism/actions/runs/34809776551) terminó con éxito en Ubuntu. Pasaron `contracts`, `compose`, `rust`, `docker-build` y `production-runtime`.

## Gate operacional

- Preflight: `S18_PREFLIGHT_OK`.
- Propietario efectivo del runner: UID `1001`; archivos efímeros con modo `0600`.
- Usuario efectivo del proceso telemetry: `10001`.
- Smoke: 3/3 frames.
- Carga real: `frames_sent=100000`, `frames_ok=100000`.
- Duración de carga reportada: `55012.881 ms`.
- Digest del dataset: `a0bed9fff13f913938f826e3cdaafd9d6de158fd3f609054b7b0969197ec630d`.
- `synthetic_cycle=false` en preflight, smoke, carga y reconnect.
- Reconnect: 3/3 frames después de stop/start del servidor.
- Artifact: `s20-linux-production-gate` con `preflight.json`, `smoke.json`,
  `telemetry.log`, `result.json` y `reconnect.json`.
- Auditoría de secretos: sin headers de claves privadas, tokens efímeros ni
  variables de autenticación en los artifacts.

## Correcciones realizadas durante el gate

1. El preflight aceptaba solo UID 0/10001 para archivos host; ahora acepta
   también el UID efectivo del runner sin cambiar el requisito de permisos
   `0600`.
2. Los secrets Compose basados en archivos no permiten remapear UID/GID en
   Compose local; el overlay production usa secrets de origen environment con
   `uid/gid=10001` y `mode=0400`, manteniendo el Compose base basado en archivos.
3. El runner dejó de usar `eval` y ejecuta Compose mediante un array Bash para
   evitar re-parsing de certificados PEM multilinea.

## Limitación fuera de alcance

No se implementó runner Linux local. El mismo gate queda reproducible mediante
`workflow_dispatch` en GitHub Actions; el runner local queda en backlog.
