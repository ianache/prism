# S18 Production Readiness Gate

## Objetivo

Convertir el flujo Docker TLS validado en S17 en un gate operacional reproducible
para CI y Linux non-root, conservando la meta de decisión de 100.000 frames.

## Estado de partida

S17 valida localmente un stream real de 100.000 payloads B2 distintos, con
`frames_ok=100000` y `synthetic_cycle=false`. El overlay production declara
UID/GID `10001:10001`, pero aún no existe una ejecución Linux automatizada que
confirme permisos de secretos, lifecycle y recuperación.

## Diseño

### CI

GitHub Actions ejecutará en un runner Linux los contratos Rust, los contratos
Python sin depender de pytest instalado globalmente, `docker compose config` para
los perfiles local/production y el build de la imagen. Los secretos y el corpus
de 100K se generarán en el runner o se inyectarán como artefactos temporales; no
se guardarán en GitHub ni en el repositorio.

### Runtime production

El job de runtime usará `docker-compose.yml` más
`docker-compose.production.yml`, secretos efímeros con permisos compatibles con
UID/GID 10001, un stream JSONL secuencial y `PRISM_ALLOW_CYCLE=false`. Validará
healthcheck, TLS/auth, 100.000 respuestas B2, digest y ausencia de procesos root.

### Resiliencia

La prueba operacional separará smoke, reinicio y drenaje. El servidor debe
observar `STARTING → READY`, aceptar tráfico, producir `DRAINING → STOPPED` ante
SIGTERM y rechazar nuevas conexiones durante el drenaje. Un cliente interrumpido
debe terminar con error identificable; un cliente nuevo debe poder reconectar y
completar un lote controlado.

### Evidencia

Cada ejecución publicará como artefacto un resumen JSON/Markdown con commit,
digest de imagen, digest del dataset, usuario efectivo, conteos, código de salida,
duración y logs sanitizados. La evidencia no incluirá tokens, claves privadas ni
el corpus completo.

## Fuera de alcance

mTLS, broker, persistencia, Kubernetes, despliegue remoto y una garantía P0 de
throughput. La meta de frames no cambia.

## Aceptación

1. CI verde en Linux para Rust, Python, Compose y Docker build.
2. Runtime production ejecutado como UID/GID 10001 sin errores de permisos.
3. TLS/auth válidos y 100.000 respuestas B2 correctas con `synthetic_cycle=false`.
4. Lifecycle y reconexión verificados con evidencia reproducible.
5. Artefacto de release sanitizado y documentación actualizada.
