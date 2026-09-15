# S21 HTTP Production Vertical Slice — evidencia

Fecha de cierre: 2026-09-15  
Rama: `work/s21-http-production-vertical-slice`  
Commit de implementación verificado: `5f01eeb`

## Resultado

S21 expone el core B0/B1/B2 mediante HTTP/1.1 acotado, con TLS, bearer auth,
health/readiness, drain/reconnect y soporte Docker Compose. TCP JSONL permanece
como protocolo por defecto.

## Verificación local

- `python -m pytest -q`: 99 passed, 21 subtests passed.
- `cargo test --workspace --release`: PASS.
- Contratos HTTP/Compose enfocados: PASS.
- `docker compose config` y overlay de producción: PASS.
- `git diff --check`: PASS.
- El objetivo comparable de throughput permanece en `100000` frames.

## Verificación GitHub Actions

Workflow: [Production Readiness run 34921551769](https://github.com/ianache/prism/actions/runs/34921551769)

Todos los jobs finalizaron correctamente: `compose`, `rust`, `contracts`,
`docker-build` y `production-runtime`.

- Gate S20 TCP/TLS/non-root: `100000/100000`, `effective_user=10001`,
  `synthetic_cycle=false`, digest `a0bed9fff13f913938f826e3cdaafd9d6de158fd3f609054b7b0969197ec630d`.
- Smoke S21 HTTP/TLS: `3/3`, `effective_user=10001`,
  `synthetic_cycle=false`, digest `9f11200afb958cd94c1f3270d3c5e843f41ac792572933cec43cb4ee28e22013`.
- Se verificaron health/readiness, procesamiento, lifecycle y reconnect.
- Los artefactos `s20` y `s21-http-production-smoke` fueron cargados por CI.

## Seguridad y límites

Los contratos y el smoke mantienen secretos fuera de respuestas y evidencia;
los contenedores permanecen non-root (`10001:10001`). El adaptador no acepta
chunked/multipart y mantiene límites estrictos de headers/body. El certificado
efímero del smoke cubre `localhost` y `server`, que es el hostname Compose.

El runner Linux local no se implementa en S21; permanece como backlog separado.
