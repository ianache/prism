# Rust S1 Protocol Completion and 100K Validation Design

**Status:** Proposed for review  
**Date:** 2026-09-12  
**Scope:** Follow-up increment after the integrated Rust B0/B1 runtime and 300-fixture smoke corpus.

## 1. Goal

Complete the Rust S1 benchmark implementation against the frozen benchmark
protocol, then validate correctness and measurement behavior on a 100,000-fixture
dataset. The 100,000 raw binary fixtures remain outside Git; Git stores only the
dataset manifest, digest, reproducibility metadata, and immutable raw JSONL
evidence selected for review.

This increment produces comparable engineering evidence. It does not claim P0
qualification, implement B2/B3, or run S2–S5.

## 2. Current gap

The integrated `prism-bench` can load the 300-fixture corpus, run B0/B1, compute
some nearest-rank percentiles, and refuse output overwrite. It does not yet
fully represent the frozen protocol: warm-up convergence, five repetitions,
all required percentile/throughput fields, complete host metadata, Rust-computed
SHA-256, workflow tax, and explicit mismatch abort semantics need to be made
observable in raw JSONL.

## 3. Design

### 3.1 Dataset preparation and immutability

The Python generator remains the independent fixture/oracle authority. A
preparation command creates a temporary 100,000-fixture directory using the
frozen seed `0x505249534D5F5631`, runs `tools.dataset.verify`, and records:

- `manifest.json`;
- `manifest.sha256`;
- fixture count and category totals;
- generator/oracle versions and command line.

The raw `fixtures/*.bin` files stay in an external artifact directory. The Rust
loader reads all bytes and expected outcomes before timing, verifies the
manifest digest using an internal standard-library SHA-256 implementation, and
keeps resident bytes in memory for measured runs. The loader rejects missing
files, digest mismatch, fixture/expected count mismatch, or changed manifest
metadata before warm-up.

### 3.2 Protocol-complete S1 runner

`RunConfig` will model the frozen S1 parameters explicitly:

- warm-up target of 10,000 frames;
- convergence windows of 1,000 frames;
- maximum warm-up iterations and a 5% p99 convergence threshold;
- five measured repetitions;
- 10,000 measured frames per repetition for S1;
- levels B0 and B1;
- scenario S1 and concurrency 1.

The runner constructs the B1 registry before timing. Each measured sample starts
with a resident payload and ends with the returned `Outcome`. Dataset I/O,
expected JSON parsing, contract loading, serialization, correctness comparison,
and raw output writing remain outside the primary timer. A mismatch aborts the
run and produces no valid performance record.

Percentiles use nearest-rank over all measured frame samples without outlier
removal: p50, p95, p99, p99.9, and maximum. Throughput is derived from total
resident frames and primary elapsed nanoseconds; MB/sec uses payload bytes.

### 3.3 Raw record schema

Each level/repetition record includes, at minimum:

- run ID, timestamp, implementation, level, scenario, concurrency;
- dataset ID, fixture count, payload-class totals, manifest SHA-256, commit,
  and exact command;
- warm-up target, convergence result, repetition number, and sample count;
- p50/p95/p99/p99.9/max nanoseconds, frames/sec, and MB/sec;
- correctness total, matches, typed rejections, and execution failures;
- CPU model, physical/logical cores, RAM, OS/kernel, Rust/compiler version,
  governor, container limits, affinity, and unavailable metrics explicitly
  marked `N/D`;
- workflow tax for matched B1/B0 p99, retaining negative values.

Raw JSONL is write-once. The harness refuses an existing output path and writes
records atomically through a temporary sibling followed by a final rename. No
hand-edited benchmark values are accepted.

### 3.4 100K validation

The 100,000-fixture run uses the same seed, generator, verifier, and protocol as
the 300-fixture smoke run. Validation occurs in this order:

1. Python generation and independent verification;
2. Rust loader digest/count verification;
3. Rust B0/B1 oracle parity over every fixture;
4. protocol-complete S1 benchmark;
5. JSONL syntax, metadata, correctness, and tax audit;
6. comparison of 300-fixture and 100K correctness, not performance claims.

The 100K binary directory is never staged or committed. Only its manifest,
digest, command metadata, and explicitly selected raw evidence may be added to
Git if they are small, immutable, and reproducible.

## 4. Interfaces

The increment extends the existing standard-library-only crates:

- `Dataset::load` returns resident fixtures, expected outcomes, digest, and
  category/payload counts;
- `RunConfig` contains explicit warm-up, convergence, repetition, and sample
  settings;
- `run_level` returns protocol-complete raw records or a typed mismatch/error;
- `percentile_nearest_rank` accepts integer nanosecond samples and supports
  p99.9 without floating-point ordering;
- `Metadata::collect` returns required host fields and `N/D` for unavailable
  metrics;
- `write_once_atomic` creates immutable raw JSONL and rejects existing paths.

No third-party Rust dependencies, broker, tracing library, transport, B2/B3, or
production workflow behavior are introduced.

## 5. Correctness and failure handling

Correctness is a hard prerequisite. Any B0/B1 mismatch against the Python
expected line, any B0/B1 disagreement, digest mismatch, missing metadata, or
invalid sample count marks the run invalid and prevents performance claims.
Typed F1 rejections remain counted separately from unexpected execution
failures. Negative workflow-tax values are serialized unchanged.

## 6. Acceptance criteria

The increment is ready when:

1. `cargo test --workspace --release` passes with the MSVC toolchain;
2. the smoke corpus remains exactly 300/300 for B0 and B1;
3. the external 100,000-fixture corpus passes Python verification and Rust
   parity for every fixture;
4. S1 emits five repetitions per level with all required percentile,
   throughput, correctness, digest, metadata, and tax fields;
5. an existing raw output is never overwritten;
6. the measured timer excludes the protocol-defined non-hot-path work;
7. no P0 qualification, B2/B3, S2–S5, broker, tracing, or transport claim is
   introduced.

## 7. Non-goals

- Committing 100,000 individual binary fixtures;
- generating or committing the 1,000,000-file release;
- changing the frozen telemetry wire format, oracle, rule table, or benchmark
  thresholds;
- adding production runtime behavior or transport;
- declaring P0 readiness from S1 or 100K evidence alone.
