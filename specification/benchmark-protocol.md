# Telemetry Benchmark Protocol Specification Map

**Status:** Specification skeleton; measurements are not comparable until blocking protocol decisions in the [gap report](gap-ambiguity-report.md) resolve.

## Dataset

Every implementation consumes identical deterministic PRISM-owned binary frames and independently generated expected outcomes. Payload classes are approximately 100, 250, and 500 bytes. Workload composition is fixed at 80% valid, 5% invalid, and 15% edge/complex. Invalid coverage includes truncation, bad magic/version, length mismatch, range violation, unsupported protocol, and checksum failure.

Fixture encoding, seed, manifest, and oracle-comparison format are `BLOCKING` as `DATA-001` in the [gap report](gap-ambiguity-report.md#blocking-decision-register). Logical frame/data semantics are mapped in [telemetry workflow](telemetry-workflow.md#frame-semantics).

## Benchmark levels and scenarios

| Level | Scope |
| --- | --- |
| B0 Native | Direct equivalent functions without PRISM abstraction. |
| B1 Workflow | PRISM contracts, registry, filter invocation, and sequential F1-F6 pipeline. |
| B2 Observable | B1 plus execution ID, per-filter timing, metrics, and tracing variants. |
| B3 Distributed | P1 producer/worker transport at workflow boundaries; never a broker between every filter. |

Scenarios are S1 single-frame latency, S2 one million sequential frames, S3 concurrency at 1/2/4/8/16/32/64 where meaningful, S4 approximately 10x burst and recovery, and S5 15–30 minute sustained load. Runs must be warmed and steady-state, cover every payload/workload class, and preserve raw results. Warm-up, repetitions, sampling, and aggregation are `BLOCKING` as `BENCH-001`.

## Timers

The primary latency timer starts when a frame is resident and ready for processing and ends when final success or typed rejection is produced. It excludes dataset I/O, process startup, contract loading, result writing, and orchestration; excluded work is reported separately.

Matched runs calculate `Workflow Tax = B1 - B0`, `Observability Tax = B2 - B1`, and `Distribution Tax = B3 - B2` using the same percentile metric and a frozen percentage formula. Timer resolution, sampling, and comparison formula are `BLOCKING` as `BENCH-001` because they are required for comparable measurements.

## Metrics

Required latency metrics are p50, p95, p99, p99.9, and max; p99 is primary and average latency is not a decision metric. Record frames/sec, MB/sec, CPU utilization, CPU ns/frame where available, RSS, heap, allocations/frame, allocated bytes/frame, memory growth, and applicable GC/JIT/context-switch data.

The authoritative gate-to-metric mapping is [acceptance criteria](acceptance-criteria.md#gate-to-metric-map). Correctness uses the independent oracle and is a hard prerequisite before performance gates can qualify a runtime.

## Reporting and reproducibility

Reports are generated from preserved machine-readable raw records; hand-edited benchmark results are forbidden. A report includes implementation, level, scenario, concurrency, payload class, percentile values, correctness count, memory metrics, gate evaluation, threats to validity, and decision-matrix input.

Comparable primary runs require CPU model, physical/logical cores, RAM, OS/kernel, runtime/compiler version, CPU governor, container limits, commit, dataset digest, and command metadata. The exact collection and normalization policy is `BLOCKING` as `BENCH-002` in the [gap report](gap-ambiguity-report.md#blocking-decision-register). JIT warm-up, garbage collection, CPU frequency variation, scheduler noise, timer resolution, compiler optimization, and allocation behavior must be reported as threats to validity.
