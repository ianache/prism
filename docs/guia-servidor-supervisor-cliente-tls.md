# Guía operativa: servidor, supervisor y cliente TLS

Esta guía describe un flujo local completo para ejecutar PRISM con TLS:

1. el servidor `prism-run` atiende conexiones TLS y solicitudes JSONL;
2. el supervisor PowerShell inicia, verifica y detiene el proceso;
3. un cliente TLS envía solicitudes B2 autenticadas.

El flujo mantiene la meta vigente de **100.000 frames** para las decisiones de capacidad y observabilidad.

## Requisitos

- Windows PowerShell.
- El binario de release `target\release\prism-run.exe`.
- Un certificado X.509 en PEM, por ejemplo `server-cert.pem`.
- La clave privada correspondiente en PEM, por ejemplo `server-key.pem`.
- Un archivo de token de una sola línea, por ejemplo `auth-token.txt`.

El archivo de token debe contener únicamente el secreto, sin imprimirlo en consola ni incluirlo en el repositorio.

Para compilar y ejecutar las pruebas del workspace:

```powershell
rtk proxy 'C:\Users\ilver\.cargo\bin\cargo.exe' test --workspace --release
```

## 1. Ejecutar el servidor directamente

Desde la raíz del repositorio:

```powershell
rtk proxy .\target\release\prism-run.exe --route b2 --listen 127.0.0.1:9000 --tls-cert .\server-cert.pem --tls-key .\server-key.pem --auth-token-file .\auth-token.txt
```

El proceso debe anunciar una línea similar a:

```text
READY 127.0.0.1:9000
```

La conexión TLS se negocia antes de la admisión S9. Una solicitud JSONL B2 requiere, como mínimo, `request_id`, `payload_hex` y `auth_token`:

```json
{"request_id":"manual-1","payload_hex":"00","auth_token":"TOKEN_SECRETO"}
```

La respuesta exitosa incluye `ok: true` y `route: "b2"`. Los errores más habituales son:

- `AUTHENTICATION_FAILED`: el token no coincide o falta.
- `INVALID_FIELD`: `payload_hex` no es una cadena.
- `INVALID_HEX`: el hexadecimal tiene longitud impar o contiene caracteres inválidos.
- `CAPACITY_EXCEEDED`: se alcanzó la capacidad configurada.
- `SERVER_DRAINING`: el servidor está cerrando conexiones nuevas.

## 2. Ejecutar con el supervisor

El supervisor S13 conserva un archivo de estado con el PID, dirección y rutas operativas. No copia el certificado, la clave ni el token dentro del estado.

Define un directorio de estado temporal:

```powershell
$state = Join-Path $env:TEMP "prism-s13-state"
```

### Arranque

```powershell
rtk proxy powershell -NoProfile -ExecutionPolicy Bypass -File .\scripts\s13_supervisor.ps1 -Action start -StateDirectory $state -Certificate .\server-cert.pem -PrivateKey .\server-key.pem -TokenFile .\auth-token.txt
```

La salida esperada contiene `S13_START_OK`, junto con la dirección y el PID del proceso.

### Verificación de salud

```powershell
rtk proxy powershell -NoProfile -ExecutionPolicy Bypass -File .\scripts\s13_supervisor.ps1 -Action check -StateDirectory $state -TokenFile .\auth-token.txt
```

La comprobación abre TLS, envía una solicitud B2 autenticada y valida la respuesta. La salida esperada contiene:

```text
S13_CHECK_OK
```

### Detención ordenada

```powershell
rtk proxy powershell -NoProfile -ExecutionPolicy Bypass -File .\scripts\s13_supervisor.ps1 -Action stop -StateDirectory $state
```

El supervisor activa el sentinel de apagado, espera el estado `STOPPED` y elimina el estado operativo. La salida esperada contiene `S13_STOP_OK`.

Si el estado apunta a un PID que ya no corresponde a un proceso `prism-run`, el supervisor rechaza la operación para evitar actuar sobre un proceso ajeno.

## 3. Cliente TLS mínimo en Python

El siguiente cliente valida el servidor contra el certificado PEM indicado. El nombre usado en `server_hostname` debe coincidir con el certificado; para un certificado emitido para `localhost`, usa `localhost`.

```python
import json
import socket
import ssl

host = "127.0.0.1"
port = 9000
token = "TOKEN_SECRETO"

context = ssl.create_default_context(cafile="server-cert.pem")

with socket.create_connection((host, port), timeout=5) as raw:
    with context.wrap_socket(raw, server_hostname="localhost") as tls:
        request = {
            "request_id": "manual-python-1",
            "payload_hex": "00",
            "auth_token": token,
        }
        tls.sendall((json.dumps(request) + "\n").encode("utf-8"))
        response = tls.makefile("rb").readline()
        print(response.decode("utf-8").rstrip())
```

La respuesta debe ser JSON y reportar `ok: true` y `route: "b2"`. En un cliente de producción, lee JSONL por líneas y no supongas que cada llamada a `recv()` contiene exactamente una respuesta.

## 4. Stack Docker Compose

El stack Compose contiene el servidor TLS y un cliente de smoke test bajo el perfil `client`. Copia `.env.example` como `.env` y ajusta las rutas de los tres archivos TLS/token:

```powershell
Copy-Item .env.example .env
New-Item .\secrets -ItemType Directory -Force
# Coloca server-cert.pem, server-key.pem y auth-token.txt dentro de .\secrets
```

Valida la configuración y arranca el servidor:

```powershell
rtk proxy docker compose --env-file .env config
rtk proxy docker compose --env-file .env up -d --build
rtk proxy docker compose --env-file .env ps
```

El servicio `server` publica `${PRISM_HOST_PORT}` en el puerto 9000 del contenedor. Su healthcheck realiza una solicitud B2 real sobre TLS con el token montado. Docker lo reinicia si el proceso termina inesperadamente.

Ejecuta el cliente dentro de la red Compose:

```powershell
rtk proxy docker compose --env-file .env --profile client run --rm client
```

La respuesta debe indicar `ok: true` y `route: "b2"`. Para revisar eventos del runner:

```powershell
rtk proxy docker compose --env-file .env logs -f server
```

Detén el stack de forma ordenada:

```powershell
rtk proxy docker compose --env-file .env down
```

El entrypoint del contenedor convierte `SIGTERM` en la creación del sentinel y permite que el runner complete `DRAINING` antes de `STOPPED`. El script completo de verificación es:

```powershell
rtk proxy powershell -NoProfile -ExecutionPolicy Bypass -File .\scripts\docker_tls_smoke.ps1 -EnvFile .env
```

No guardes `.env`, certificados ni tokens en Git. Usa rutas absolutas o rutas bajo `secrets/` controladas localmente y conserva los montajes como solo lectura.

## 5. Ingestión de tramas de telemetría

El perfil `telemetry` lee archivos binarios desde `PRISM_INPUT_PATH`, los envía como `payload_hex` y valida una respuesta B2 por cada trama. Para una prueba pequeña con los fixtures válidos del repositorio:

```powershell
rtk proxy docker compose --env-file .env --profile telemetry up -d --build server
rtk proxy docker compose --env-file .env --profile telemetry run --rm `
  -e PRISM_FRAME_TARGET=4 `
  -e PRISM_INPUT_GLOB="p0-valid-*.bin" telemetry-client
```

Para la corrida objetivo de 100.000 frames, configura un directorio que contenga al menos esa cantidad de archivos válidos:

```powershell
$env:PRISM_INPUT_PATH = "./datasets/telemetry"
$env:PRISM_INPUT_GLOB = "*.bin"
$env:PRISM_FRAME_TARGET = "100000"
rtk proxy docker compose --env-file .env --profile telemetry run --build --rm telemetry-client
```

El cliente imprime un resumen con `S15_TELEMETRY_OK`, `frames_sent`, `frames_ok` y `dataset_sha256`. El smoke automatizado es:

```powershell
rtk proxy powershell -NoProfile -ExecutionPolicy Bypass -File .\scripts\docker_telemetry_smoke.ps1 -EnvFile .env
```

El directorio de entrada se monta como solo lectura. El cliente falla si hay menos archivos que `PRISM_FRAME_TARGET`, si una respuesta no conserva su `request_id` o si el servidor devuelve `ok: false`.

Para una prueba sintética de capacidad que reutiliza un conjunto pequeño de tramas válidas, activa explícitamente `PRISM_ALLOW_CYCLE=true`. Este modo no debe confundirse con un corpus de 100.000 tramas distintas; el resumen lo marca como `synthetic_cycle: true`.

`PRISM_BATCH_SIZE` controla cuántas solicitudes se envían antes de leer sus respuestas. El valor predeterminado es `1`; para una prueba de capacidad ordenada puede usarse `64` sin abrir conexiones adicionales.

Para un corpus real reproducible, genera un único stream secuencial fuera de Git:

```powershell
python scripts/generate_telemetry_stream.py `
  --template tests/fixtures/p0-smoke/fixtures/p0-valid-0000060-105.bin `
  --output C:\temp\prism\telemetry.jsonl `
  --manifest C:\temp\prism\telemetry.json `
  --frames 100000
```

Configura `PRISM_INPUT_PATH`, `PRISM_INPUT_STREAM=/data/frames/telemetry.jsonl`,
`PRISM_FRAME_TARGET=100000`, `PRISM_ALLOW_CYCLE=false` y `PRISM_BATCH_SIZE`.
El cliente reporta `synthetic_cycle=false` y el digest de payloads.

El overlay `docker-compose.production.yml` fija UID/GID `10001:10001` para un
runtime Linux non-root. En Docker Desktop Windows puede usarse explícitamente
`PRISM_CONTAINER_USER=0:0` por la adaptación local de permisos.

## 6. Prueba manual TLS incluida

Para ejecutar la comprobación operativa S12:

```powershell
rtk proxy powershell -NoProfile -ExecutionPolicy Bypass -File .\scripts\manual_s12_tls.ps1 -Certificate .\server-cert.pem -PrivateKey .\server-key.pem -TokenFile .\auth-token.txt
```

La salida esperada es similar a:

```text
MANUAL_S12_TLS_OK route=b2 ok=True
```

Este script acepta explícitamente el certificado local mediante un callback de validación para facilitar la prueba manual. Esa excepción no debe copiarse a un cliente remoto de producción: allí se debe validar la cadena de confianza y el nombre del servidor correctamente.

## Resolución rápida de problemas

| Síntoma | Revisión |
|---|---|
| No aparece `READY` | Verifica que existan el binario, el certificado, la clave y el token; revisa el puerto y stderr. |
| `AUTHENTICATION_FAILED` | Confirma que el token enviado coincide exactamente con la única línea de `auth-token.txt`. |
| `INVALID_HEX` | Usa una cadena hexadecimal de longitud par, por ejemplo `00` o `deadbeef`. |
| `CAPACITY_EXCEEDED` | Reduce la concurrencia de la prueba o espera la recuperación de capacidad antes de reintentar. |
| `SERVER_DRAINING` | No abras conexiones nuevas durante la detención; espera `STOPPED` y arranca de nuevo. |
| Estado del supervisor inexistente | Ejecuta `start` antes de `check` o `stop`, usando el mismo `StateDirectory`. |
| Falla de confianza TLS en Python | Asegura que el certificado se pase como `cafile` y que `server_hostname` coincida con su SAN/CN. |

## Alcance actual

Este flujo cubre TLS de servidor, autenticación por token, JSONL B2, capacidad, lifecycle y supervisión local. Todavía no incluye mTLS, rotación automática de certificados, broker, persistencia, servicio Windows, despliegue remoto ni Kubernetes.
