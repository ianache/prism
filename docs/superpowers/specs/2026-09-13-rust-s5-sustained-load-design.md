# Rust S5 Sustained Load Design

**Status:** Proposed for review  
**Date:** 2026-09-13  
**Scope:** Rust B0/B1/B2 follow-up after the S4 burst evidence.

## Goal

Measure whether the Rust B0, B1, and B2 paths remain correct, bounded, and
stable under sustained load. S5 extends the existing benchmark with resource
sampling and a time series of fixed 100,000-frame windows; it does not select
a P0 runtime by itself.

## Constraints

- Use protocol v1.1 and exactly 100,000 measured frames in every complete
  decision window.
- Use five repetitions for every level. The complete B0/B1/B2 run receives a
  900-second total sustained-load budget, divided equally across three levels
  and five repetitions (60 seconds per level/repetition).
- Keep B0, B1, B2 behavior, F1-F6 order, oracle comparison, and existing S1-S4
  contracts unchanged.
- Use the same resident external 100K corpus for every level and repetition.
- Keep dataset I/O, window scheduling, resource sampling, aggregation,
  serialization, and writing outside the primary processing timer.
- Use Rust standard library APIs for timing and Windows process metrics; do not
  add a benchmark framework, broker, transport, or external telemetry service.
- Keep binary fixtures outside Git and make raw JSONL output write-once.
- Do not implement B3, queues, network transport, persistent backpressure, or
  multi-hour soak testing in this increment.

## Run model

The CLI accepts `--scenario S5`, `--levels b0,b1,b2`, `--samples 100000`,
`--repetitions 5`, and `--duration-seconds 900`. The duration is a total wall
clock budget divided equally among the five repetitions. Within each
repetition, the runner repeatedly processes complete 100,000-frame windows in
deterministic corpus order until the repetition budget ends. A partial final
window is discarded from the comparable output and reported as an incomplete
tail.

Each complete window has a monotonically increasing `window_index` and retains
the level, repetition, measured frame count, latency percentiles, throughput,
correctness counters, and resource snapshots taken immediately before and
after the window. The runner records process RSS and the platform-specific
metrics that are available without changing the timed path. Unavailable values
are serialized as `N/D`, never guessed.

The five repetitions run sequentially for each level. Level ordering is
`b0`, `b1`, `b2`; this ordering is orchestration only and is excluded from the
primary timer. Every level must produce at least one complete window per
repetition for the package to be valid.

## Stability and resource metrics

Latency uses nearest-rank p50, p95, p99, p99.9, and maximum over each complete
window. Throughput is calculated from the measured processing interval, not
from the wall-clock repetition budget. The audit computes first-to-last p99
change per level and repetition, RSS growth from the first to last complete
window, and the count of incomplete tails.

The S5 evidence package supplies inputs to the provisional gates:

- correctness is 100 percent for every complete window;
- RSS growth is below 10 percent where RSS is available;
- p99 increase from the first to last complete window is at most 10 percent;
- workflow and observability taxes remain paired by repetition and window.

Failure of any correctness, identity, frame-count, finite-metric, ordering,
or resource-series invariant rejects the package. A missing optional platform
metric is valid only when represented explicitly as `N/D`; missing required
latency, throughput, or correctness evidence is invalid.

## Output contract

Raw JSONL records are window records rather than one aggregate row per
repetition. Each record contains `scenario: "S5"`, `level`, `repetition`,
`window_index`, `measured_frames: 100000`, `duration_seconds`, latency and
throughput metrics, correctness counters, resource snapshots, protocol and
dataset identity, host metadata, and B2 F1-F6 evidence where applicable.

The output path must not already exist. The audit requires the complete tuple
set `(level, repetition)` for all three levels and five repetitions, strictly
increasing window indices within each tuple, and equal 100K frame counts.
Taxes are matched by level, repetition, and window index: B1 against B0 for
workflow tax and B2 against B1 for observability tax. Negative taxes remain
valid.

## Verification and evidence

The increment is complete when:

1. CLI and runner contracts prove the 100K window, five-repetition, duration,
   deterministic ordering, incomplete-tail, and resource-sampling behavior.
2. The full Rust suite and all Python audit suites pass.
3. The independent S5 audit accepts valid multi-window evidence and rejects
   missing tuples, partial windows, duplicated indices, non-finite metrics,
   identity mismatches, correctness failures, and unpaired taxes.
4. An external package documents the command, dataset digest, host metadata,
   duration, window series, resource availability, gate calculations, and
   limitations.
5. No S5 binary fixture is tracked and no B3, broker, transport, or P0
   qualification claim is introduced.
