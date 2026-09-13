# Rust S4 Burst Design

**Status:** Proposed for review  
**Date:** 2026-09-13  
**Scope:** Rust B0/B1/B2 follow-up after the integrated S3 concurrency evidence.

## Goal

Measure how the Rust B0, B1, and B2 paths behave across a stable baseline, an
approximately 10× burst, and recovery to the baseline rate, using the same
100,000-frame workload unit in every phase.

S4 is a controlled engineering experiment. It does not introduce a production
queue, broker, transport, backpressure subsystem, or P0 qualification claim.

## Constraints

- Use protocol v1.1 and exactly 100,000 frames in each phase.
- Use five repetitions for every level and phase.
- Keep Rust B0/B1/B2 functional behavior, telemetry format, oracle, and F1–F6 order unchanged.
- Use the same resident external 100K corpus for baseline, burst, and recovery.
- Keep dataset loading, arrival schedule creation, correctness comparison, aggregation, serialization, and writing outside the primary processing timer.
- Keep concurrency as an explicit S3-compatible input; the first S4 package uses concurrency 1 to isolate burst behavior from S3 scaling.
- Use Rust standard library only.
- Keep binary fixtures outside Git and make raw JSONL output write-once.
- Do not implement S5 sustained load, brokers, network transport, persistent queues, or cross-language runners.

## Phase model

Before the measured phases, the harness runs a calibration B0 sequence with
100,000 frames and five repetitions at concurrency 1. It takes the median of
the five measured B0 throughputs as the service-capacity reference. The
baseline offered rate is exactly 80% of that median, leaving a declared margin
for stable recovery. The burst offered rate is exactly 10× the baseline rate.
The harness records the calibration throughputs, selected median, rates, and
nanosecond inter-arrival intervals in the package metadata.

Each repetition processes three ordered phases:

1. `baseline`: 100,000 frames at the calibrated baseline arrival interval.
2. `burst`: 100,000 frames at one-tenth of the calibrated interval, representing exactly 10× offered arrival rate.
3. `recovery`: 100,000 frames at the calibrated baseline interval after the burst.

The scheduler creates the phase arrival timestamps before timing. The runner
feeds resident frames in deterministic corpus order and records whether each
frame started before or after its scheduled arrival. The primary processing
timer covers only frame processing; scheduler bookkeeping and phase report
assembly remain outside it.

Because the selected design is synchronous and has no queue, S4 reports
offered-rate pressure and processing lateness; it does not claim queue depth,
backpressure correctness, or loss behavior. A later queue experiment would be
a separate increment.

## Output contract

The raw JSONL package contains 45 records: 3 levels × 3 phases × 5
repetitions. Each record includes `scenario: "S4"`, `phase`, `concurrency`,
`repetition`, `measured_frames: 100000`, offered frames/sec, processed
frames/sec, lateness counts/percentiles, latency percentiles, throughput,
correctness counters, protocol metadata, dataset identity, and B2 F1–F6
evidence.

The phase names are exactly `baseline`, `burst`, and `recovery`. A valid
package contains one record for every `(level, phase, repetition)` tuple and
uses the same dataset digest and host metadata for all records.

## Metrics and failure handling

Latency percentiles use nearest-rank p50, p95, p99, p99.9, and maximum without
outlier removal. Throughput uses the measured processing interval. Lateness is
computed against the prebuilt arrival schedule and must retain zero values for
frames processed on time.

Workflow tax compares B1 p99 to B0 p99 within the same phase and repetition.
Observability tax compares B2 p99 to B1 p99 within the same phase and
repetition. Negative tax values remain valid.

Any correctness mismatch, invalid phase order, missing phase tuple, worker
failure, non-finite metric, dataset mismatch, or invalid frame count rejects
the package. Existing output paths are never overwritten.

## Verification and evidence

The increment is complete when:

1. Focused contracts prove phase order, 100K frames per phase, deterministic
   schedule construction, lateness accounting, and phase/repetition tax pairing.
2. The full Rust and Python suites pass.
3. The independent S4 audit accepts exactly 45 records and rejects malformed
   phase, repetition, identity, metric, correctness, and tax cases.
4. The external package documents command, digest, host metadata, phase
   results, limitations, and explicit non-qualification.
5. No S4 binary fixture is tracked and no queue, broker, S5, or P0 claim is
   introduced.
