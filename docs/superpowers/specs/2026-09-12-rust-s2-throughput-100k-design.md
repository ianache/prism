# Rust S2 Throughput 100K Design

**Status:** Proposed  
**Date:** 2026-09-12  
**Scope:** Next benchmark increment after B2 local observability.

## Goal

Measure sequential throughput and tail latency for B0, B1, and B2 using the
normative 100,000-frame decision baseline, producing independently auditable
evidence without introducing concurrency or transport.

## Protocol

S2 uses protocol v1.1:

- 100,000 measured frames per repetition;
- five repetitions for each level;
- 10,000 warm-up frames;
- consecutive 1,000-frame convergence windows;
- p99 convergence threshold of 5%;
- nearest-rank p50, p95, p99, p99.9, and maximum;
- identical external 100K corpus for B0, B1, and B2.

The previous 1,000,000-frame S2 target is retired. No S2 result may use a
different frame target without a new versioned protocol decision.

## Levels and comparisons

- B0 is the direct native baseline.
- B1 is the sequential PRISM pipeline and provides the workflow-tax baseline.
- B2 is B1 with `local_metrics` observability and provides the observability-tax comparison.

The CLI must execute B0, B1, and B2 in one matched invocation when tax values
are requested. B2 is not a valid standalone decision run because it lacks the
matched B1 p99 baseline.

## Measurements

Each raw record includes latency percentiles, elapsed time, frames/sec, MB/sec,
correctness totals, dataset digest, protocol parameters, metadata, and the
appropriate tax matched by repetition. B2 records additionally include the
execution ID, `local_metrics` variant, and F1–F6 counters/timings.

The primary timer covers resident frame processing. Dataset loading, oracle
comparison, aggregation, serialization, and writing remain outside the timer.
For B2, observer callbacks are inside the timed processing path because their
cost is the measurement target.

## Correctness and audit

Every level must match the expected JSONL oracle for all 100,000 measured
frames in every repetition. The independent S2 audit must reject incomplete
level sets, wrong repetitions, wrong frame counts, digest mismatches,
non-finite metrics, correctness mismatches, and incorrect workflow or
observability tax arithmetic.

Raw output is immutable and write-once. B0/B1/B2 records may be emitted in one
file, but the audit must group them by level and repetition before comparing
matched p99 values.

## Acceptance criteria

S2 is accepted when:

1. B0, B1, and B2 each have repetitions 1–5.
2. Every record reports exactly 100,000 measured frames.
3. All correctness comparisons pass.
4. Throughput and tail metrics are finite and present.
5. Workflow tax is computed B1 against B0 per repetition.
6. Observability tax is computed B2 against B1 per repetition.
7. The independent audit accepts the complete raw set and rejects malformed fixtures.
8. Existing B0/B1/B2 contracts remain green.
9. No P0 qualification, concurrency, broker, transport, or durable-execution claim is made.

## Non-goals

- S3 concurrency testing.
- S4 burst/recovery testing.
- S5 sustained-load testing.
- B3 transport or broker evaluation.
- New runtime implementations in Go or Java.
- Reinstating the 1,000,000-frame target.
- Hand-edited benchmark records or reports.

## Risks and limitations

The corpus is replayed sequentially and does not establish concurrent scaling,
queue behavior, recovery, or long-duration stability. Host scheduling,
frequency variation, timer resolution, and instrumentation overhead remain
threats to validity and must be reported with the raw evidence.
