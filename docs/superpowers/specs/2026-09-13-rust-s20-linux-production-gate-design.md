# Rust S20 Linux Production Gate Closure — Design

Fecha: 2026-09-13  
Estado: aprobado para planificación  
Base: S19 (`96621e6`)

## Objetivo

Cerrar la única brecha operacional pendiente de S18/S19 ejecutando en un
runner Ubuntu el stack Docker Compose real con TLS, usuario no-root,
lifecycle/reconnect y aceptación de exactamente 100000 frames. El resultado
debe quedar en artefactos sanitizados y en evidencia versionada antes de
promocionar los cambios a `main`.

## Alcance

Incluye:

- publicar la corrección de contratos S19 y disparar el workflow de readiness;
- hacer que el workflow sea invocable manualmente y que identifique sus
  artefactos como S20;
- ejecutar y verificar los jobs Rust, contratos, Compose, build y
  `production-runtime`;
- conservar `100000` como meta obligatoria de decisión;
- validar TLS, non-root `10001:10001`, smoke, procesamiento real sin ciclo
  sintético, stop/start y reconnect;
- revisar que los artefactos no contengan secretos y documentar el resultado;
- integrar en `main` y publicar únicamente si todos los gates pasan.

Fuera de alcance para S20: instalar o mantener un runner Linux local. Se
registrará como backlog explícito para una iteración posterior.

## Diseño técnico

El workflow `.github/workflows/production-readiness.yml` será la única puerta
de aceptación. Se añadirá `workflow_dispatch` para permitir una ejecución
manual reproducible además de `push` y `pull_request`. El job
`production-runtime` invocará el script existente con
`--frames 100000 --batch-size 64`, usando secretos y corpus efímeros creados
por el script; sus salidas irán a `artifacts/s20` y el artifact se llamará
`s20-linux-production-gate`.

La implementación reutilizará `scripts/docker_production_readiness.sh` y
`scripts/docker_production_preflight.py`. Solo se modificarán las aserciones o
la instrumentación que una ejecución Ubuntu demuestre necesarias; no se
relajarán los criterios para ocultar fallos. El runner local Windows se usará
solo para validaciones estáticas y de contratos.

## Flujo y aceptación

1. `contracts` pasa scanner y contratos S14/S15/S17/S18.
2. `compose` produce configuración válida base y production.
3. `docker-build` construye `server` y `telemetry-client`.
4. `production-runtime` pasa preflight, smoke de 3 frames, carga de 100000
   frames, lifecycle, reconnect y cleanup.
5. El resultado reporta `frames_sent=100000`, `frames_ok=100000` y
   `synthetic_cycle=false`; el preflight reporta el contrato non-root esperado.
6. El artifact contiene solo JSON/logs sanitizados, sin PEM, tokens ni rutas
   sensibles. La evidencia S20 enlaza el run, commit, jobs y hashes relevantes.

Ante cualquier fallo, el incremento queda parcial: se conserva el artifact de
diagnóstico, se registra la causa y no se mezcla ni publica como release.

## Verificación

- Local: contratos S14–S18, suite Rust release, compilación Python, scanner,
  `docker compose config` y `git diff --check`.
- CI Ubuntu: workflow completo y artifact de `production-runtime`.
- Auditoría: revisión de contenido del artifact y actualización de
  `docs/consumo.md`.

## Backlog posterior

Crear un runner Linux local reproducible para ejecutar el mismo script sin
depender de GitHub Actions, con documentación de instalación y paridad de
variables. No forma parte del criterio de cierre de S20.
