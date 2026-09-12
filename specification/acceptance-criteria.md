# Telemetry Acceptance Criteria

**Status:** Provisional specification gates. Their metric definitions become actionable only when blocking protocol decisions in the [gap report](gap-ambiguity-report.md) resolve.

## Measurement preconditions

All gates apply only to matched runs using the dataset, scenario, timer, raw-result, and metadata rules in [benchmark protocol](benchmark-protocol.md). Candidates consume the same bytes, follow ordered F1-F6, and compare logical outcomes through the independent oracle mapped in [telemetry workflow](telemetry-workflow.md).

Correctness is a hard prerequisite: a runtime with any logical mismatch against the oracle cannot qualify on performance.

## Gate-to-metric map

| Profile | Gate | Threshold or required outcome | Metric definition and evidence |
| --- | --- | --- | --- |
| P0 | Latency p50 | <= 250 microseconds | `p50` primary-timer latency from matched warmed runs; [metrics](benchmark-protocol.md#metrics) and [timers](benchmark-protocol.md#timers). |
| P0 | Latency p95 | <= 500 microseconds | `p95` primary-timer latency from matched warmed runs; [metrics](benchmark-protocol.md#metrics). |
| P0 | Latency p99 | <= 1 millisecond | `p99` primary-timer latency from matched warmed runs; [metrics](benchmark-protocol.md#metrics). |
| P0 | Logical correctness | 100% expected benchmark outputs | Oracle comparison count over every fixture/class; [dataset](benchmark-protocol.md#dataset). Hard prerequisite. |
| P0 | Memory | Bounded, no sustained growth | RSS, heap, allocation, allocated-bytes/frame, and memory-growth series during S5; [metrics](benchmark-protocol.md#metrics). |
| P0 | Sustained stability | No material latency degradation | p50/p95/p99 trend across S5 intervals, interpreted by `QOS-001`; [scenarios](benchmark-protocol.md#benchmark-levels-and-scenarios). |
| P0 | Workflow tax | B1 p99 tax <= 20% versus matched B0 | Matched primary-timer p99 values using `Workflow Tax = B1 - B0` and frozen percentage formula; [timers](benchmark-protocol.md#timers). Provisional target. |
| P1 | Latency p50 | <= 10 milliseconds | `p50` primary-timer latency for P1 transport evaluation; [metrics](benchmark-protocol.md#metrics). |
| P1 | Latency p95 | <= 25 milliseconds | `p95` primary-timer latency for P1 transport evaluation; [metrics](benchmark-protocol.md#metrics). |
| P1 | Latency p99 | <= 50 milliseconds | `p99` primary-timer latency for P1 transport evaluation; [metrics](benchmark-protocol.md#metrics). |
| P1 | Queue and backpressure | Bounded queue and explicit backpressure | Scenario evidence and transport metrics from B3; [scenarios](benchmark-protocol.md#benchmark-levels-and-scenarios). |
| P1 | Reliability | No message loss under declared durability semantics; recovery tested | Raw correctness/loss count and recovery evidence; [reporting](benchmark-protocol.md#reporting-and-reproducibility). |
| P1 | Throughput | Documented | Frames/sec and MB/sec from B3 scenarios; [metrics](benchmark-protocol.md#metrics). |

## Qualification rule

A P0 candidate qualifies only if every P0 row is satisfied after correctness passes. P1 gates are later transport-evaluation gates and do not authorize a P0 runtime or broker. P2-P4 compatibility concerns are `NON-BLOCKING` because they do not affect current P0 bytes, expected output, or comparable P0/P1 measurements.
