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

The evidence package does not claim P0, B2/B3, or S2–S5 qualification. Host
resource values that cannot be collected are emitted as `N/D`.
