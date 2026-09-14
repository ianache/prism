# S19 — CI Contract Remediation & Linux Gate Closure

Fecha: 2026-09-13  
Rama: `work/s19-ci-gate`

## Alcance

S19 corrige los contratos que bloquearon el primer workflow de S18 y deja
preparado el gate Linux para su nueva ejecución en GitHub Actions.

## Diagnóstico

El workflow `34804714412` falló únicamente en `contracts` por dos expectativas
obsoletas:

- El contrato S14 buscaba `- client` y mounts TLS directos, mientras que el
  Compose vigente usa `profiles: ["client"]` y Compose secrets montados bajo
  `/run/secrets/`.
- El contrato S18 buscaba el texto `forbidden`, pero el scanner define el
  identificador `FORBIDDEN_MARKERS`, cuyo cambio a `forbidden_markers` evita
  además el auto-match del propio contrato.

Los jobs `rust` y `compose` del mismo workflow habían pasado; los jobs
dependientes de `contracts` no se ejecutaron.

## Cambios

- Actualizado `tests/test_s14_compose_contract.py` para validar profiles,
  Compose secrets y rutas `/run/secrets/`.
- Actualizado `tests/test_s18_ci_contract.py` para validar
  `forbidden_markers`.
- Renombrado el marcador interno del scanner en `scripts/ci_contract.py` y
  conservada la detección de material secreto.

## Verificación local

- Contratos S14/S15/S17/S18: `14 passed`.
- El gate Linux real, incluyendo runtime non-root, lifecycle y aceptación de
  100000 frames, queda pendiente de la ejecución del workflow en un runner
  Ubuntu. El host Windows no dispone de `/bin/bash`/WSL funcional para
  reproducirlo localmente.

## Criterio de cierre

S19 queda implementado y verificable localmente para contratos. La aceptación
operacional completa S18 se cerrará cuando GitHub Actions ejecute
`production-runtime` con éxito.
