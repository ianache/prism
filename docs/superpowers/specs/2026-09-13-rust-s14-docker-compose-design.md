# S14 Docker Compose Operational Stack

## Objetivo

Proveer un stack local reproducible para ejecutar `prism-run` con TLS y token,
supervisarlo mediante Docker Compose y probarlo con un cliente TLS bajo demanda.
La meta operativa de capacidad permanece en 100.000 frames.

## Alcance

- Imagen multi-stage para el binario release `prism-run`.
- Servicio `server` con configuración por variables de entorno.
- Certificado, clave y token montados como archivos de solo lectura.
- Healthcheck real sobre TLS + JSONL B2 autenticado.
- Servicio `client` bajo el perfil `client`, ejecutable con `docker compose run`.
- Script de apagado ordenado que crea el sentinel al recibir `SIGTERM`.
- Documentación, validación estructural y smoke test reproducible.

## No alcance

No se añaden mTLS, rotación automática de certificados, broker, persistencia,
servicio Windows, Kubernetes, despliegue remoto ni un supervisor PowerShell
dentro del contenedor.

## Arquitectura

`server` ejecuta `prism-run --route b2 --listen ...` en la red Compose. El
entrypoint recibe las variables, crea el sentinel bajo `/run/prism` y delega
señales al runner; ante `SIGTERM` crea el sentinel para activar el lifecycle
`DRAINING`/`STOPPED`. Compose reinicia el contenedor si el proceso falla y usa
un healthcheck TLS autenticado para marcar la disponibilidad.

`client` reutiliza una imagen Python pequeña y se habilita solo con el perfil
`client`. Lee el token y la CA desde montajes de solo lectura, conecta al
servicio por nombre DNS `server` y envía una solicitud B2 JSONL.

## Configuración

Variables no secretas en `.env.example`:

- `PRISM_BIND`: dirección interna, por defecto `0.0.0.0:9000`.
- `PRISM_ROUTE`: ruta, por defecto `b2`.
- `PRISM_WORKERS`: workers, por defecto `2`.
- `PRISM_CONNECTION_QUEUE`: cola, por defecto `2`.
- `PRISM_DRAIN_TIMEOUT_MS`: timeout de drenaje, por defecto `5000`.
- `PRISM_HOST_PORT`: puerto publicado, por defecto `9000`.
- `PRISM_TLS_CERT`, `PRISM_TLS_KEY`, `PRISM_AUTH_TOKEN`: rutas host de secretos.

Los valores secretos no se colocan en `.env.example`; Compose los monta desde
las rutas configuradas y el contenedor solo recibe rutas internas.

En Docker Desktop para Windows, los secretos locales se presentan con permisos
root-only; el Compose de desarrollo usa `user: "0:0"` para que el runner pueda
leerlos. Un despliegue Linux/Kubernetes debe reemplazar esta adaptación por
permisos de secreto no root antes de producción.

## Criterios de aceptación

1. `docker compose config` resuelve sin variables obligatorias faltantes.
2. La imagen compila el workspace release y arranca `prism-run`.
3. El healthcheck rechaza TLS/token incorrectos y acepta una respuesta B2 válida.
4. `docker compose --profile client run --rm client` devuelve `ok=true` y `route=b2`.
5. `docker compose down` permite al runner atravesar `DRAINING` y finalizar.
6. La documentación explica instalación, certificados, arranque, verificación,
   cliente, logs, actualización y solución de problemas.
