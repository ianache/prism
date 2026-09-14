# S16 Docker Runtime Validation & Telemetry Acceptance

## Resultado

Validar el stack Compose real con Docker Desktop, TLS, autenticación, healthcheck,
ingestión B2 y lifecycle. La meta de decisión es 100.000 frames.

## Decisiones operativas

- Los secretos locales se modelan como Compose `secrets`.
- Docker Desktop Windows requiere `user: "0:0"` para leer esos secretos root-only;
  esta adaptación no se considera configuración de producción Linux/Kubernetes.
- El cliente conserva orden en una conexión TLS y permite batching configurable.
- `PRISM_ALLOW_CYCLE=true` es exclusivamente sintético: reutiliza fixtures válidos
  y marca `synthetic_cycle=true` en la salida.
- La ruta por defecto es B2 y el healthcheck ejecuta una solicitud B2 real.

## Aceptación verificada

1. Imagen multi-stage construida con Docker Desktop.
2. Smoke TLS de 4 tramas válido.
3. Lifecycle `STARTING → READY → DRAINING → STOPPED` observado en logs.
4. Carga sintética de 100.000 solicitudes con 100.000 respuestas `ok`.
5. Dataset de 100.000 archivos distintos identificado como ineficiente sobre bind
   mount Windows; no se reclama como evidencia de throughput físico de disco.
