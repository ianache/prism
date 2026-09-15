# S23 WSL2 Local Gate Enablement — evidencia

Fecha: 2026-09-15  
Rama: `work/s23-wsl2-local-gate-enablement`

## Resultado

La preparación del host fue verificada sin instalar componentes. WSL está
disponible y Docker Desktop responde (`29.7.2`); Compose responde (`v5.5.1`).
El host solo expone las distribuciones internas `docker-desktop` y
`docker-desktop-data`, que no son operativas para este gate. Por ello los
cuatro casos locales quedan explícitamente como `NO EJECUTADA`.

## Matriz local

| Caso | Comando abstracto | Estado | Duración | frames | Usuario efectivo | Digest | Motivo |
|---|---|---|---|---:|---|---|---|
| tcp smoke | `run_production_gate.ps1 -Protocol tcp -Frames 3 -BatchSize 1 -EvidenceDir <tcp-smoke>` | NO EJECUTADA | N/D | N/D | N/D | N/D | No existe una distribución Linux de usuario en WSL. |
| http smoke | `run_production_gate.ps1 -Protocol http -Frames 3 -BatchSize 1 -EvidenceDir <http-smoke>` | NO EJECUTADA | N/D | N/D | N/D | N/D | No existe una distribución Linux de usuario en WSL. |
| tcp 100000 | `run_production_gate.ps1 -Protocol tcp -Frames 100000 -BatchSize 64 -EvidenceDir <tcp-100000>` | NO EJECUTADA | N/D | N/D | N/D | N/D | Requiere una distro Linux de usuario, Bash y Docker Compose accesibles. |
| http 100000 | `run_production_gate.ps1 -Protocol http -Frames 100000 -BatchSize 64 -EvidenceDir <http-100000>` | NO EJECUTADA | N/D | N/D | N/D | N/D | Medición adicional; requiere una distro Linux de usuario. |

Los smoke reducidos no sustituyen la decisión comparable. Cuando el host esté
preparado, el gate TCP solo será aceptado con `frames_sent=100000` y
`frames_ok=100000`; HTTP `100000` será únicamente una medición adicional.

## Diagnóstico del host

- `wsl.exe --status`: WSL disponible.
- `wsl.exe -l -v`: solo `docker-desktop` y `docker-desktop-data`; ambas quedan
  excluidas por el launcher.
- `docker.exe version`: servidor Docker Desktop accesible.
- `docker.exe compose version`: Compose accesible.
- `bash --version`: verificación documentada para ejecutar dentro de la distro
  operativa, no evidencia de una distro local disponible.
- El launcher devuelve `no Linux distribution is installed` sin instalar nada.

La evidencia operativa equivalente en Linux permanece en S20/S21 mediante
GitHub Actions. Este documento distingue la disponibilidad del host de la
ejecución del runtime y no afirma un PASS local inexistente.

## Seguridad y criterios preservados

No se registran PEM, tokens, credenciales ni valores de variables de
autorización.
Los comandos usan placeholders para rutas de evidencia. El flujo conserva
TCP como default, HTTP explícito, `PRISM_ALLOW_CYCLE=false` y
`PRISM_CONTAINER_USER=10001:10001`. La meta comparable permanece exactamente
`100000` frames.

## Verificación

- Contratos S23 de evidencia: PASS.
- Ejecución local de los cuatro casos: `NO EJECUTADA` por diagnóstico ambiental
  determinista.
- Escaneo dirigido de secretos en `docs/evidence`: PASS.
