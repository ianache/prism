# Rust S3 Concurrency Design

**Status:** Proposed for review  
**Date:** 2026-09-12  
**Scope:** Follow-up increment after the integrated Rust S2 sequential-throughput evidence.

## Goal

Measure the concurrency behavior of the Rust B0, B1, and B2 implementations
over the same external 100,000-frame corpus, using deterministic worker
partitioning and matched repetition/concurrency taxes.

S3 produces engineering evidence for scalability. It does not claim P0
qualification, production readiness, or that concurrency improves latency.

## Constraints

- Use protocol v1.1 and the fixed target of 100,000 measured frames per repetition.
- Run exactly five measured repetitions for each level and concurrency.
- Use concurrency values 1, 2, 4, 8, 16, 32, and 64 workers.
- Run B0, B1, and B2 over the identical dataset and invocation semantics.
- Keep the frozen telemetry format, oracle, filter order, and correctness rules unchanged.
- Use Rust standard library only; do not add a broker, transport, tracing dependency, or external runtime.
- Keep dataset loading, contract construction, correctness comparison, aggregation, serialization, and output writing outside the primary measured interval.
- Keep binary 100K fixtures outside Git; commit only small reproducibility metadata and raw evidence where appropriate.

## Execution model

For each level, concurrency, and repetition:

1. The parent validates the configuration and prepares resident dataset bytes.
2. The parent creates exactly `concurrency` scoped workers.
3. Worker `w` processes frame indexes satisfying `index % concurrency == w`.
4. Each B1/B2 worker owns its pipeline; each B2 worker owns its observer collector.
5. A standard-library barrier synchronizes worker start and completion.
6. The primary elapsed interval starts immediately before the synchronized processing phase and ends after all workers complete.
7. Workers return their local samples, counters, and B2 evidence to the parent.
8. The parent merges results in worker/index order, validates correctness, computes percentiles, and writes output outside the timed interval.

The fixed worker-per-repetition model avoids persistent pool state across
repetitions. It makes warm-up, ownership, and teardown explicit, at the cost
of measuring worker creation and coordination at each repetition. That cost is
part of S3 concurrency behavior and must be reported, not hidden.

## Metrics and semantics

Every record identifies `level`, `scenario: "S3"`, `concurrency`, and
`repetition`. It retains p50, p95, p99, p99.9, maximum latency, frames/sec,
MB/sec, correctness counters, protocol metadata, and B2 F1–F6 evidence.

Latency samples represent per-frame processing elapsed time collected by the
workers. Throughput uses the synchronized wall-clock interval for the complete
worker group. Percentiles use nearest-rank without outlier removal.

Taxes are matched on both concurrency and repetition:

- B1 workflow tax = `(B1 p99 - B0 p99) / B0 p99 * 100`.
- B2 observability tax = `(B2 p99 - B1 p99) / B1 p99 * 100`.

Negative taxes remain valid evidence. Missing baselines or a baseline p99 of
zero produce `0.0` and are rejected by the independent audit when the S3 set
is incomplete.

## Output contract

One JSONL package contains 105 records: 3 levels × 7 concurrency values × 5
repetitions. Records are immutable and the output path is write-once.

The audit requires exactly one record for every tuple:

`(level ∈ {b0,b1,b2}, concurrency ∈ {1,2,4,8,16,32,64}, repetition ∈ {1..5})`.

It also requires identical dataset identity, protocol version, measured frame
count, correctness totals, finite nonnegative throughput, and valid tax
pairing. B2 records must contain all F1–F6 keys and nonnegative invocation
counts. No audit result is a P0 qualification.

## Failure handling

Any worker panic, join failure, unexpected execution failure, correctness
mismatch, invalid sample count, missing tuple, non-finite metric, or dataset
identity mismatch invalidates the run. The harness must not publish a valid
S3 package after such a failure. Existing output files must never be
overwritten.

## Verification and evidence

The increment is complete when:

1. Focused concurrency contracts prove deterministic partitioning, worker
   ownership, synchronized timing, complete tuples, and matched taxes.
2. The complete Rust workspace and Python audit suites pass.
3. The external S3 package passes JSON parsing and the independent audit with
   105 records and 100,000 frames per record.
4. The report documents host metadata, dataset digest, command, throughput
   scaling, latency behavior, limitations, and explicit non-qualification.
5. No S3 binary fixture is tracked and no S4/S5 or production transport work
   is introduced.

## Non-goals

- Persistent worker pools or asynchronous runtimes.
- Burst, sustained-load, queue, backpressure, broker, or network tests (S4/S5 or P1 work).
- Cross-language comparisons.
- Automatic P0 selection or qualification.
- Changes to B0/B1/B2 functional behavior.
