# Rust B0/B1 y benchmark S1

El workspace usa solo la biblioteca estándar y contiene `prism-runtime` y
`prism-bench`. B0 es la ruta directa; B1 valida un registro estático F1–F6.
Ambas rutas se comparan contra el oráculo Python en el corpus smoke de 300
fixtures y en el corpus externo protocolario de 100.000 fixtures.

## Verificación

```text
python -m tools.dataset.generate --output tests/fixtures/p0-smoke --seed 0x505249534D5F5631 --count 300
python -m tools.dataset.verify --dataset tests/fixtures/p0-smoke
cargo test --workspace --release
```

### S5 stability remediation

The 2026-09-13 diagnostic rerun preserved the 100,000-frame target and added
window boundary timestamps outside the timed filter path. Correctness and RSS
passed, but p99 stability remained above the provisional 10% gate. See
`docs/evidence/rust-s5-stability-remediation-2026-09-13.md` for the unresolved
root-cause hypotheses and explicit non-qualification.

## S1

The S1 records already produced with 10,000 measured frames are historical
evidence. Starting with protocol v1.1 and B2, every comparable decision uses
100,000 measured frames per repetition; a different target requires a new
versioned protocol decision.

El corpus se carga antes de medir. La salida JSONL se escribe una sola vez y
se rechaza si ya existe:

```text
cargo run -p prism-bench --release -- \
  --dataset tests/fixtures/p0-smoke --levels b0,b1 --scenario S1 \
  --warmup 10000 --samples 10000 --repetitions 5 \
  --output results/raw/s1.jsonl \
  --cpu-model <model> --cores <cores> --ram-bytes <bytes> \
  --os <os> --governor <governor> --affinity <affinity>
```

## S4

S4 mide Rust B0/B1/B2 en tres fases ordenadas: `baseline`, `burst` y
`recovery`, con 100.000 frames por fase y cinco repeticiones. La calibración
B0 determina la capacidad de referencia; baseline usa 80% de la mediana y
burst usa 10× baseline. El paquete final tendrá 45 registros y no constituye
calificación P0.

La medición primaria usa `Instant` alrededor del procesamiento de bytes
residentes; I/O, carga de contratos, serialización, comparación y escritura
quedan fuera. Los percentiles son nearest-rank.

## Validación externa de 100K

Los binarios individuales se generan fuera del repositorio. El manifiesto, su
digest y la salida JSONL son los artefactos auditables; no se versiona el
corpus binario:

```text
powershell -ExecutionPolicy Bypass -File scripts/prepare-rust-100k.ps1
python scripts/audit-rust-s1.py \
  --dataset D:\02-PERSONAL\TOOLS\prism-datasets\p0-100k \
  --raw D:\02-PERSONAL\TOOLS\prism-datasets\p0-100k-s1.jsonl
```

After changing the runner or raw-record schema, write a new JSONL path (raw
outputs are immutable) and audit that new path. The audit is intentionally
limited to the external 100K corpus; for the 300-fixture smoke output, parse
each JSONL line with `python -c "import json,sys; [json.loads(line) for
line in open(sys.argv[1], encoding='utf-8') if line.strip()]" <raw.jsonl>`.

La prueba Rust de conteo es opt-in mediante `PRISM_100K_DATASET`; sin esa
variable el contrato no carga el corpus pesado. La auditoría S1 exige diez
filas (cinco repeticiones para B0 y B1), digest coincidente, corrección total
y tax de workflow numérico.

La ejecución validada produjo 10 filas, 100.000 fixtures y digest
`269eb27cdeed982f81cb0ecb82e2c279014a19fa67bf998ccf529a681ea145c4`.

S1 es evidencia de ingeniería y no constituye una calificación P0. El release
externo de 1,000,000 de archivos individuales permanece pendiente y fuera de
Git.

## B2

B2 usa la misma meta v1.1 de 100.000 frames por repetición. Auditar una salida
B2 contra su baseline B1:

```text
python scripts/audit-rust-b2.py --baseline <b1.jsonl> --raw <b2.jsonl>
```

La auditoría exige cinco repeticiones, campos F1–F6 completos, corrección,
protocolo v1.1 y tax de observabilidad calculado por repetición.

The evidence package does not claim P0, B2/B3, or S2–S5 qualification. Host
resource values that cannot be collected are emitted as `N/D`.

## S5

S5 ejecuta B0/B1/B2 en cinco repeticiones secuenciales. El presupuesto total
predeterminado es de 900 segundos, dividido entre tres niveles y cinco
repeticiones (60 segundos por nivel/repetición), y emite ventanas completas de
exactamente 100.000 frames. La
ventana final incompleta se informa y no se incluye en la evidencia comparable.

```text
cargo run -p prism-bench --release -- \
  --dataset D:\\02-PERSONAL\\TOOLS\\prism-datasets\\p0-100k \
  --levels b0,b1,b2 --scenario S5 --concurrency 1 \
  --warmup 10000 --samples 100000 --repetitions 5 --duration-seconds 900 \
  --output D:\\02-PERSONAL\\TOOLS\\prism-datasets\\p0-100k-s5.jsonl \
  --cpu-model model --cores cores --ram-bytes ram \
  --os windows --governor governor --affinity affinity
python scripts/audit-rust-s5.py \
  --dataset D:\\02-PERSONAL\\TOOLS\\prism-datasets\\p0-100k \
  --raw D:\\02-PERSONAL\\TOOLS\\prism-datasets\\p0-100k-s5.jsonl
```

La auditoría valida estabilidad de p99, crecimiento RSS cuando está
disponible, corrección total, identidad del dataset y taxes emparejados por
nivel, repetición y ventana. S5 no constituye calificación P0 ni implementa
B3, colas, brokers o transporte.

S6 reuses the S5 protocol shape and adds optional process/system CPU
diagnostics outside the timed path. Its immutable output is identified as an
S6 root-cause-isolation package and remains engineering evidence only.

## S3

S3 measures Rust B0/B1/B2 with five repetitions of 100,000 frames at
concurrencies `1, 2, 4, 8, 16, 32, 64`. The combined command emits 105 records
and does not claim P0 qualification:

```text
cargo run -p prism-bench --release -- \
  --dataset D:\02-PERSONAL\TOOLS\prism-datasets\p0-100k \
  --levels b0,b1,b2 --scenario S3 --concurrency 1,2,4,8,16,32,64 \
  --warmup 10000 --samples 100000 --repetitions 5 \
  --output D:\02-PERSONAL\TOOLS\prism-datasets\p0-100k-s3.jsonl \
  --cpu-model <model> --cores <cores> --ram-bytes <bytes> \
  --os <os> --governor <governor> --affinity <affinity>
```
# S7: flujo vertical local

El binario `prism-run` soporta un flujo funcional continuo local por stdin/stdout:

```powershell
Get-Content .\s7-input.jsonl | .\target\release\prism-run.exe --route b1 --request-id-prefix demo
```

Cada línea de entrada es un sobre JSONL con `request_id` y `payload_hex`, donde
`payload_hex` contiene el frame binario canónico. Cada línea de salida conserva
el identificador, la ruta, `ok` y el resultado serializado. Las líneas inválidas
se informan en banda y no detienen las siguientes; EOF termina correctamente.

Las rutas son B0 (directa), B1 (pipeline) y B2 (pipeline con observer F1–F6).
Este slice es local y funcional: no implica transporte de red, broker, métricas
de benchmark, calificación P0 ni reinterpreta la evidencia S5/S6.

## S8: transporte TCP local

Para probar el mismo flujo sobre TCP, inicia el listener en loopback:

```powershell
rtk proxy .\target\release\prism-run.exe --route b2 --listen 127.0.0.1:9000 --request-id-prefix tcp
```

Envía las mismas líneas JSONL desde un cliente TCP. El proceso anuncia la
dirección efectiva por stderr (`LISTENING ...`), procesa varias solicitudes por
conexión y acepta otra conexión después de un EOF limpio. Las líneas mayores de
64 KiB cierran solo la conexión actual.

Para habilitar concurrencia acotada entre clientes:

```powershell
rtk proxy .\target\release\prism-run.exe --route b2 --listen 127.0.0.1:9000 --workers 2 --connection-queue 2 --request-id-prefix s9
```

Cada conexión conserva su propio orden. Si los workers y la cola están llenos,
la conexión adicional recibe `CAPACITY_EXCEEDED` y se cierra; el listener sigue
activo y recupera capacidad cuando un cliente termina.

## S10: lifecycle y shutdown controlado

Inicia el listener con un archivo-sentinel administrado externamente:

```powershell
rtk proxy .\target\release\prism-run.exe --route b2 --listen 127.0.0.1:9000 --shutdown-file .\shutdown.flag --drain-timeout-ms 5000
```

Con `--shutdown-file`, el proceso informa `STARTING`, `READY`, `DRAINING` y `STOPPED` por stderr. Para
solicitar apagado, crea el archivo desde otra terminal:

```powershell
New-Item .\shutdown.flag -ItemType File
```

El runner no elimina el sentinel. Deja de admitir trabajo nuevo, drena las
conexiones existentes y termina dentro del timeout configurado.

## S11: transporte TLS y autenticación por token

El TCP local puede protegerse con un certificado PEM y una clave privada PEM;
ambos deben existir y se validan antes de `READY`:

```powershell
rtk proxy .\target\release\prism-run.exe --route b2 --listen 127.0.0.1:9000 `
  --tls-cert .\server-cert.pem --tls-key .\server-key.pem `
  --auth-token-file .\auth-token.txt
```

El handshake TLS ocurre antes de admitir la conexión al pool. Si se configura
`--auth-token-file`, cada sobre JSONL debe incluir `auth_token`; un token
ausente o incorrecto devuelve `AUTHENTICATION_FAILED` en banda y la conexión
puede continuar. El token nunca se imprime ni se incluye en una respuesta.

La configuración es opcional: sin esas opciones se conserva el flujo TCP
plaintext de S8–S10. S11 no incluye rotación de certificados, mTLS, despliegue
remoto ni calificación P0.

La verificación de proceso TLS se ejecuta con:

```powershell
rtk proxy 'C:\Users\ilver\.cargo\bin\cargo.exe' test -p prism-runner --release --test s12_tls_e2e
```

Para una prueba manual con certificados PEM existentes:

```powershell
rtk proxy powershell -NoProfile -ExecutionPolicy Bypass -File .\scripts\manual_s12_tls.ps1 `
  -Certificate .\server-cert.pem -PrivateKey .\server-key.pem -TokenFile .\auth-token.txt
```

## S13: supervisión operativa local

El supervisor PowerShell administra estado no secreto, readiness, health check
y apagado controlado:

```powershell
$state = Join-Path $env:TEMP "prism-s13-state"
rtk proxy powershell -NoProfile -ExecutionPolicy Bypass -File .\scripts\s13_supervisor.ps1 `
  -Action start -StateDirectory $state -Certificate .\server-cert.pem `
  -PrivateKey .\server-key.pem -TokenFile .\auth-token.txt
rtk proxy powershell -NoProfile -ExecutionPolicy Bypass -File .\scripts\s13_supervisor.ps1 `
  -Action check -StateDirectory $state -TokenFile .\auth-token.txt
rtk proxy powershell -NoProfile -ExecutionPolicy Bypass -File .\scripts\s13_supervisor.ps1 `
  -Action stop -StateDirectory $state
```

`start` devuelve `S13_START_OK`, `check` devuelve `S13_CHECK_OK` y `stop`
devuelve `S13_STOP_OK`. El wrapper no registra tokens ni material de
certificados y no instala un servicio Windows.
