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

S5 ejecuta B0/B1/B2 en cinco repeticiones secuenciales. Cada repetición usa
un presupuesto de 180 segundos dentro de una duración total predeterminada de
900 segundos y emite ventanas completas de exactamente 100.000 frames. La
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
