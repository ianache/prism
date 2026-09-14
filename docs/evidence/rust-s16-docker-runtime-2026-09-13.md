# Evidencia S16: Docker Runtime Validation & Telemetry Acceptance

Fecha: 2026-09-13

## Preflight

- Docker Desktop Linux: PASS, server `29.7.2`, 4 CPUs, 2.841 GiB.
- Dataset inicial: 300 fixtures binarios.
- Certificado y clave PEM autofirmados, más token efímero, creados fuera del repositorio.

## Resultados runtime

| Flujo | Resultado |
|---|---|
| Compose config con perfil telemetry | PASS |
| Build multi-stage Docker | PASS |
| Smoke TLS de 4 frames | PASS; `frames_sent=4`, `frames_ok=4` |
| Healthcheck TLS + B2 | PASS; servicio `Healthy` |
| Lifecycle | PASS; `STARTING → READY → DRAINING → STOPPED` |
| Aceptación 100K sintética | PASS; `frames_sent=100000`, `frames_ok=100000` |

## Digest

La corrida sintética produjo:

```text
dataset_sha256=16bd4ae6d36ef2d6ba4c152f4ee88a95343941e9769fde05b71a7be4ce116018
synthetic_cycle=true
S15_DOCKER_TELEMETRY_OK
```

Las 100.000 solicitudes reutilizaron fixtures válidos mediante
`PRISM_ALLOW_CYCLE=true`; no representan 100.000 archivos distintos.

## Incidencias corregidas

- Secretos root-only en Docker Desktop Windows: Compose usa `user: "0:0"` como
  adaptación local explícita.
- `TERM` interrumpía `wait` antes del drain: el entrypoint ahora continúa esperando
  al runner después de crear el sentinel.
- El cliente no aceptaba el prefijo operativo `req-` del runner.
- Smoke no reconstruía la imagen del cliente: ahora usa `run --build`.
- Se añadió batching ordenado para evitar un round-trip por frame.

## Limitación

La enumeración de 100.000 archivos individuales sobre un bind mount Windows fue
demasiado lenta; por eso la aceptación 100K usa el modo sintético explícito. Esto
valida protocolo, TLS, lifecycle y capacidad de solicitudes, no throughput de
almacenamiento ni una corrida P0 distinta.
