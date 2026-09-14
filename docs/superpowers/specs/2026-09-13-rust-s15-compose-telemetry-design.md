# S15 Compose Telemetry Ingestion 100K

## Objetivo

Ejecutar un flujo reproducible de ingestión de tramas binarias desde un dataset
montado hasta `prism-run` B2 mediante TLS, token y JSONL, conservando la meta de
100.000 frames.

## Diseño

Compose mantiene `server` como servicio TLS B2 y añade `telemetry-client` bajo
el perfil `telemetry`. El cliente lee archivos binarios ordenados desde
`PRISM_INPUT_PATH`, convierte cada trama a hexadecimal, envía un sobre JSONL
por conexión TLS y verifica cada respuesta. La concurrencia por defecto es 1
para conservar orden y facilitar evidencia determinista.

El dataset se monta como solo lectura. Certificado y token siguen montados como
solo lectura y no se incluyen en la imagen. El cliente devuelve código distinto
de cero si no puede alcanzar el objetivo, recibe una respuesta no válida o
detecta un error de protocolo.

## Alcance y límites

Incluye lectura de archivos binarios, envío JSONL B2, TLS, autenticación,
conteo, digest del conjunto procesado y ejecución Compose. No incluye broker,
persistencia, reintentos semánticos, ingestión desde dispositivos ni
concurrencia distribuida.

## Aceptación

1. Compose resuelve el perfil `telemetry` y sus variables.
2. Un cliente puede procesar un directorio de fixtures con orden determinista.
3. El cliente envía hasta `PRISM_FRAME_TARGET` frames y falla si no alcanza el objetivo.
4. Cada respuesta conserva el `request_id` enviado, permitiendo el prefijo operativo que añade el runner, y debe indicar `ok=true`.
5. El contrato automatizado cubre configuración, cliente y salida resumida.
6. La documentación explica smoke pequeño y corrida de 100.000 frames.
