# Rust B0/B1 Runtime and S1 Benchmark Design

**Status:** Proposed for implementation planning  
**Date:** 2026-09-12  
**Scope:** First executable Rust increment after the frozen telemetry specification and deterministic Python oracle.

## 1. Goal

Implement comparable Rust B0 and B1 telemetry paths and a reproducible S1 benchmark harness. B0 establishes the direct native baseline. B1 implements the PRISM F1-F6 contract pipeline and makes its workflow tax measurable against B0.

The increment proves correctness and measurement boundaries. It does not claim P0 qualification, which requires the complete release dataset and a controlled benchmark environment.

## 2. Architecture

Use a Cargo workspace with two crates:

```text
Cargo.toml
crates/prism-runtime/
  src/lib.rs
  src/model.rs
  src/crc32c.rs
  src/frame.rs
  src/rules.rs
  src/b0.rs
  src/b1.rs
crates/prism-bench/
  src/main.rs
tests/fixtures/p0-smoke/
results/raw/
```

`prism-runtime` contains the integer-only wire model, frame checks, rules, B0, and B1. `prism-bench` owns dataset loading, warm-up, timing, correctness comparison, metadata collection, and raw JSONL output. The Python oracle remains an independent authority and is never imported or invoked inside the measured Rust path.

Use Rust's standard library only for this increment. Do not add a benchmark framework, broker, tracing library, or production runtime dependency before the measurement boundaries are validated.

## 3. B0 and B1 boundaries

Both levels accept resident `&[u8]` and return a common outcome enum:

- `Normalized` with integer telemetry, ordered sensors, classification, severity, and route;
- `Rejected` with one of the seven stable F1 codes, stage, and typed context;
- `ExecutionFailure` for unexpected faults, counted separately from typed rejections.

B0 calls direct native-equivalent functions without registry or contract abstraction. B1 uses explicit F1-F6 filter functions and a statically registered sequential pipeline. B1 registry descriptors are validated at startup, outside the primary timer.

Only shared constants, wire types, and CRC primitives may be common to B0 and B1. Validation/pipeline behavior remains separately expressed so B0 is a meaningful native baseline.

F1 preserves the frozen order: truncation, magic, version, length, protocol, ranges, checksum. F1 rejection stops the pipeline; F2-F6 are not invoked after a rejection.

## 4. Data representation and memory

All fields use the frozen integer units and little-endian layout from `specification/telemetry-workflow.md`. Sensor storage is bounded for 76 records. The hot path borrows input bytes and uses fixed-capacity storage so B0/B1 do not allocate on each invocation.

Dataset I/O, expected-result parsing, contract loading, JSON serialization, and raw-result writing occur outside the primary timer. The timer starts with resident bytes and ends at the Rust outcome.

## 5. Smoke corpus and correctness

Version a small immutable 300-fixture corpus under `tests/fixtures/p0-smoke/`, containing raw binary fixtures and canonical expected-result JSONL produced by the Python generator/oracle. It covers all payload classes, valid/invalid/edge_complex categories, F1 mutation precedence, threshold equality, sensor absence, and boundary values.

Rust integration tests compare B0 and B1 outcomes with the expected results and with each other. A mismatch fails the run before any performance result can be considered valid.

The benchmark CLI accepts any compatible external dataset through `--dataset`; the smoke corpus is only the first correctness and development fixture.

## 6. S1 benchmark harness

The CLI supports:

```text
cargo run -p prism-bench -- \
  --dataset tests/fixtures/p0-smoke \
  --levels b0,b1 \
  --scenario S1 \
  --warmup 10000 \
  --samples 10000 \
  --repetitions 5 \
  --output results/raw/<run-id>.jsonl
```

Use a monotonic nanosecond clock. Warm-up and measured iterations use the protocol's S1 boundaries. Percentiles use nearest-rank. Raw records include implementation, level, scenario, concurrency, payload class, p50/p95/p99/p99.9/max, throughput, correctness counts, memory fields when available, commit, dataset digest, command, run ID, and host/runtime metadata.

The harness must not hand-edit or overwrite an existing raw output. Reports and gate evaluation are separate follow-up work generated only from raw records.

## 7. Testing strategy

- Unit tests for CRC-32C, frame offsets, length classes, ranges, sensor ordering, F1 precedence, rule thresholds, and route outputs.
- Integration tests for B0/B1 equivalence over the 300-fixture smoke corpus.
- CLI tests for missing dataset, invalid level/scenario, existing output, and correctness mismatch.
- Full `cargo test --workspace` before any benchmark run.
- A benchmark run is invalid if any correctness count mismatches or required metadata is absent.

## 8. Acceptance criteria

The increment is ready when:

1. `cargo test --workspace` passes on a clean machine.
2. B0 and B1 match the Python oracle for every smoke fixture and match each other.
3. The S1 harness emits valid immutable JSONL raw records with the required metadata and percentile fields.
4. The primary timer excludes I/O, startup, contract loading, JSON, and result writing.
5. No B2/B3, broker, tracing, production workflow, or P0 qualification claim is present.

## 9. Non-goals and follow-up

This increment does not generate or commit the 1,000,000-file release, implement B2/B3, measure S2-S5, add transport, or select a final P0 runtime. Those require separate plans after B0/B1 correctness and S1 output are reviewed.
