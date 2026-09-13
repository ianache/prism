# Telemetry Benchmark Protocol

**Status:** Frozen v1.1 protocol.

**Protocol version:** 1.1

## Dataset and benchmark levels

All implementations consume identical raw binary fixtures and expected JSONL results from datasets/. Workload composition is 80 percent valid, 5 percent invalid, and 15 percent edge/complex. Payload classes are 105, 249, and 501 bytes.

| Level | Definition |
| --- | --- |
| B0 | direct native equivalent functions without PRISM abstraction |
| B1 | PRISM contracts, registry, and sequential F1-F6 pipeline |
| B2 | B1 plus execution id, per-filter timing, metrics, and tracing variant |
| B3 | producer/worker transport at workflow boundaries for future P1 evaluation |

## Timer and taxes

The primary timer starts with a resident frame ready for processing and ends at the final success or typed rejection. Dataset I/O, startup, contract loading, result writing, and orchestration are excluded and reported separately. Use a monotonic nanosecond clock.

Matched percentile values define tax_percent(A,B) = (B-A)/A*100. Workflow Tax = (B1 p99 - B0 p99) / B0 p99 * 100, with equivalent formulas for B2/B1 and B3/B2. Negative values are retained and reported, not clamped.

## Run protocol

Warm until 10,000 frames have run and two consecutive 1,000-frame windows differ by no more than 5 percent in p99. Then execute five measured repetitions of 100,000 measured frames per repetition for every comparable decision, including S2 and B2. Percentiles use nearest-rank over all measured frame samples; no outlier removal. Historical S1 evidence using 10,000 measured frames remains valid as historical evidence only. S3 tests concurrency 1/2/4/8/16/32/64 where meaningful, S4 uses baseline then approximately 10x burst and recovery, and S5 runs 15-30 minutes. Any exception to the 100,000-frame target requires a new versioned protocol decision.

Every comparable run includes all payload classes and workload categories. A run is invalid if required metadata or correctness records are missing.

## Required metrics and metadata

Record p50, p95, p99, p99.9, max, frames/sec, MB/sec, CPU utilization, CPU ns/frame where available, RSS, heap, allocations/frame, allocated bytes/frame, memory growth, and applicable GC/JIT/context-switch data. Metadata must include CPU model, physical/logical cores, RAM, OS/kernel, runtime/compiler, governor, container limits, commit, dataset digest, command, run id, and affinity.

## Reporting

Raw machine-readable records are immutable inputs. Reports are generated from raw records and contain gate evaluation, threats to validity, and decision-matrix input. Hand-edited benchmark results are forbidden.

Threats include JIT warm-up, GC, frequency variation, scheduler noise, timer resolution, compiler optimization, and allocation behavior.
