# Rust S21 — HTTP Production Vertical Slice

Fecha: 2026-09-14  
Estado: aprobado para especificación detallada  
Base: `main` en `a5ec97a`  
Meta de decisión: `100000` frames permanece obligatoria para benchmarks y gates comparables.

## Objetivo

Exponer el flujo validado B0/B1/B2 mediante una interfaz HTTP/1.1 ejecutable
localmente y en Docker Compose, conservando el transporte TCP JSONL existente.
El incremento debe permitir que un cliente HTTP procese un sobre de telemetría,
consulte salud/readiness y observe un apagado controlado sin duplicar la lógica
del núcleo de procesamiento.

## Motivación y prioridad

S20 cerró el gate productivo en Ubuntu mediante GitHub Actions. Un runner Linux
local queda como backlog operativo, pero no aumenta directamente la superficie
consumible del sistema. HTTP sí agrega una interfaz de integración para clientes
web, servicios internos y pruebas manuales, por lo que se prioriza como S21.

## Alcance

Incluye:

- modo HTTP/1.1 configurable para el binario `prism-run`;
- preservación del modo TCP JSONL como comportamiento por defecto y sin cambios
  incompatibles;
- `POST /v1/process` con un sobre JSON `{request_id, payload_hex}`;
- selección de ruta B0/B1/B2 mediante la configuración del servidor, no por
  datos arbitrarios enviados por el cliente;
- límite estricto de cuerpo equivalente al límite de entrada existente de 64 KiB;
- respuestas JSON con códigos HTTP estables y el mismo resultado de dominio que
  el flujo TCP;
- autenticación HTTP mediante `Authorization: Bearer <token>` cuando el token
  esté configurado, sin incluir el token en logs ni respuestas;
- reutilización de TLS rustls, secretos Compose y usuario non-root `10001:10001`;
- `GET /healthz` para liveness y `GET /readyz` para readiness;
- respuesta `503` durante `DRAINING` o antes de que el listener esté listo;
- integración del modo HTTP con shutdown, drain timeout, workers y capacidad
  existentes;
- contratos automatizados, pruebas de proceso plaintext/TLS, documentación y
  smoke en Compose.

## Fuera de alcance

- HTTP/2, WebSockets, SSE y streaming de cuerpos;
- solicitudes chunked o multipart;
- colas, brokers, persistencia o almacenamiento de resultados;
- cambio de semántica B0/B1/B2 o del protocolo de frames;
- benchmark P0 HTTP o sustitución de la meta de `100000` frames;
- instalación automática de WSL2, Docker Desktop o un runner Linux local;
- exposición pública, rate limiting distribuido o gateway inverso.

## Diseño técnico

### Frontera de transporte

`prism-run` añadirá un modo de protocolo HTTP configurable. TCP JSONL seguirá
siendo el valor por defecto para mantener S7–S20. El modo HTTP usará la misma
configuración de ruta, workers, cola, TLS, autenticación y lifecycle; solo
cambiará la adaptación entrada/salida en la frontera de transporte.

La ruta se fija al iniciar el servidor (`b0`, `b1` o `b2`). No se permitirá que
el cliente HTTP cambie la ruta por request, evitando que una API de producción
altere el contrato operativo del proceso.

### Contrato HTTP

`POST /v1/process`

- Requiere `Content-Type: application/json`.
- Acepta un cuerpo JSON con `request_id` y `payload_hex`.
- Requiere `Authorization: Bearer <token>` si el servidor tiene token configurado.
- Devuelve `200` cuando el sobre es válido y el resultado de dominio puede ser
  serializado, incluso si B0/B1/B2 produce una decisión negativa del dominio.
- Devuelve `400` para JSON, campos, hex o tamaño inválidos.
- Devuelve `401` para autenticación ausente o incorrecta.
- Devuelve `503` si el servidor está drenando o sin capacidad disponible.
- Devuelve `500` solo para una falla interna no atribuible al request.

La respuesta conserva `request_id`, `route`, `ok` y el resultado serializado
actual. Los códigos de error de dominio permanecen en el JSON; los códigos HTTP
representan la frontera de transporte.

`GET /healthz`

- Devuelve `200` y una respuesta JSON mínima mientras el proceso está vivo.
- No requiere autenticación y no ejecuta B0/B1/B2.

`GET /readyz`

- Devuelve `200` cuando el listener acepta trabajo nuevo.
- Devuelve `503` durante `DRAINING`, antes de `READY` o después de iniciar el
  cierre.
- No requiere autenticación y no expone secretos.

### Seguridad y límites

El parser HTTP será estricto y acotado: método, path, headers, `Content-Length`,
tipo de contenido y cuerpo tendrán límites explícitos. Se rechazará `Transfer-
Encoding: chunked` en S21 para mantener un contrato pequeño y auditable. TLS
seguirá usando los certificados y claves ya cargados por `security::load`.

La extracción de Bearer se mantendrá separada del envelope JSON. El núcleo
recibirá una credencial ya normalizada para que TCP y HTTP compartan la misma
decisión de autenticación sin introducir el token en la salida.

### Docker Compose

El servicio `server` podrá arrancar en modo `tcp` o `http` mediante una variable
de entorno explícita, preservando `tcp` como default. El mismo overlay de
producción seguirá proporcionando TLS, secrets environment-backed y usuario
`10001:10001`. El healthcheck se alineará con `/healthz` cuando el modo HTTP
esté activo; el flujo TCP existente seguirá usando su comprobación actual.

## Verificación y aceptación

1. La suite Rust existente continúa verde en release.
2. Los contratos HTTP cubren método/path, content type, límite, JSON inválido,
   autenticación, códigos HTTP y forma de respuesta.
3. Un proceso real plaintext completa `/healthz`, `/readyz` y `POST /v1/process`
   para B0, B1 y B2.
4. Un proceso real TLS completa el mismo flujo con token correcto y rechaza el
   token ausente/incorrecto sin filtrar secretos.
5. El flujo HTTP respeta `DRAINING`, `SERVER_DRAINING` y el timeout de cierre.
6. Compose base y production generan configuración válida en ambos modos.
7. El smoke documentado procesa al menos 3 frames HTTP; los gates comparables
   de rendimiento conservan exactamente `100000` frames y no se reinterpretan.
8. La auditoría de artifacts no encuentra PEM, tokens ni variables secretas.
9. La guía manual incluye ejemplos PowerShell y `curl` para plaintext/TLS.

## Riesgos y decisiones

- No se añadirá un framework HTTP completo salvo que el parser acotado resulte
  insuficiente durante la implementación; cualquier cambio de dependencia debe
  justificarse en el plan y en la revisión de seguridad.
- El listener HTTP no reutilizará texto JSONL como protocolo externo; reutilizará
  el núcleo de dominio y adaptará el envelope una sola vez.
- HTTP no reemplaza TCP en S21. La migración de clientes queda fuera de alcance.

## Resultado esperado

Al finalizar S21, un operador podrá levantar el stack Compose, consultar salud y
readiness, enviar una trama B0/B1/B2 mediante HTTP plaintext o TLS y observar un
apagado seguro, con contratos y evidencia reproducibles. El runner Linux local
queda explícitamente pospuesto para una iteración posterior.
