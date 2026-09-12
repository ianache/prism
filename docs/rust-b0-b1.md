# Rust B0/B1 y benchmark S1

El workspace usa solo la biblioteca estándar y contiene `prism-runtime` y
`prism-bench`. B0 es la ruta directa; B1 valida un registro estático F1–F6.
Ambas rutas se comparan contra el oráculo Python en 300 fixtures inmutables.

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

S1 es evidencia de ingeniería y no constituye una calificación P0. El release
externo de 1,000,000 de archivos individuales permanece pendiente y fuera de
Git.
