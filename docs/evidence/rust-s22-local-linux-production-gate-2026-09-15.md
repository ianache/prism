# S22 Local Linux Production Gate — evidencia

Fecha: 2026-09-15  
Rama: `work/s22-local-linux-production-gate`

## Matriz local

| Caso | Estado | Observación |
|---|---|---|
| tcp smoke | NO EJECUTADA | No existe una distribución Linux de usuario en WSL. |
| http smoke | NO EJECUTADA | No existe una distribución Linux de usuario en WSL. |
| tcp 100000 | NO EJECUTADA | Requiere WSL/Linux con Bash y Docker Compose accesibles. |
| http 100000 | NO EJECUTADA | Requiere WSL/Linux con Bash y Docker Compose accesibles. |

## Diagnóstico

El host expone WSL y Docker Compose, pero únicamente las distribuciones
internas `docker-desktop` y `docker-desktop-data`. El launcher las excluye y
termina con diagnóstico accionable; no instala componentes ni inicia una
ejecución incompleta. El gate Linux equivalente permanece validado por
GitHub Actions en S20/S21.

La ausencia de una distribución Linux local no es un `FAIL` del gate de
producción; si una distribución válida está instalada y el gate falla, el
estado se registra como `FAIL` junto con su código de salida y evidencia.

## Criterios preservados

- El launcher mantiene TCP como default y permite seleccionar HTTP.
- La meta comparable continúa siendo exactamente `100000` frames.
- `PRISM_ALLOW_CYCLE=false` y `PRISM_CONTAINER_USER=10001:10001` permanecen
  en el flujo existente.
- No se incluyen PEM, tokens ni secretos efímeros en esta evidencia.
