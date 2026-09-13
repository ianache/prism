# Rust S6 P99 Root-Cause Isolation 100K Design

**Status:** Proposed for review  
**Date:** 2026-09-13  
**Scope:** Host/process telemetry and controlled diagnosis of the S5 p99 stability failure.

## Goal

Identify whether the S5 p99 regression is associated with scheduler/OS noise,
CPU-time or frequency/thermal drift, resource growth, or measurement
orchestration, while preserving the 100,000-frame decision target and all
B0/B1/B2 processing semantics.

## Current evidence

The published S5 baseline and the remediation rerun both pass correctness and
RSS growth checks but fail the provisional p99 stability gate. The remediation
adds ordered window timestamps and reports unmatched tails, yet does not
isolate a unique cause. S6 therefore adds host/process telemetry rather than
changing the benchmark gate or filter implementation.

## Constraints

- Keep protocol v1.1, exactly 100,000 measured frames per complete window, five repetitions, and 900 seconds total.
- Keep concurrency at 1 and preserve B0/B1/B2 behavior, percentile calculation, and tax pairing.
- Capture diagnostics outside the primary filter timer.
- Preserve all raw observations; do not smooth, discard outliers, change the 10% p99 threshold, or selectively rerun failures.
- Use optional `N/D` values when a Windows API or host metric is unavailable.
- Do not add B3, brokers, queues, transports, production runtime behavior, or third-party dependencies.
- Use a new immutable external output path and never commit raw JSONL or binary fixtures.

## Diagnostic model

Each complete window records boundary snapshots containing the existing RSS and
wall-clock fields plus process CPU time, system CPU time, and enough elapsed
time to derive process-CPU/wall-time utilization. Host metadata records the
effective OS, processor count, affinity, governor/power-plan value, and
availability status of optional metrics. The timed B0/B1/B2 path is unchanged.

The report pairs rows by `(level, repetition, window_index)` and shows raw p99,
first-to-last p99 change, CPU deltas, CPU/wall ratio, RSS deltas, timestamp
position, and unmatched tails. It classifies evidence as supporting,
contradicting, or insufficient for each hypothesis; it never replaces the
provisional p99 gate with a derived correlation score.

## Interfaces

- `ResourceSnapshot` gains optional process/system CPU nanoseconds and a
  monotonic sample timestamp.
- `SustainedWindow` and `RawRecord` serialize those fields with stable names
  and explicit nanosecond units.
- `audit-rust-s5.py` validates nonnegative numeric diagnostics, ordered
  boundaries, and identity while retaining historical baseline compatibility.
- `report-rust-s5.py` emits per-level/repetition diagnostic tables and
  hypothesis conclusions.
- A comparison script requires matching dataset/protocol/frame/repetition/
  host identity and distinct raw digests.

## Failure and portability behavior

Unavailable telemetry is represented as `N/D` and produces an `UNAVAILABLE`
diagnostic dimension, not an integrity failure. Malformed values, negative
durations, duplicate identities, missing common windows, or protocol mismatch
remain hard audit failures. A p99 or RSS threshold failure remains a reported
gate failure, not a malformed-package failure.

## Verification and acceptance

- Rust contracts cover snapshot ordering, optional metrics, serialization, and unchanged 100K windows.
- Python contracts cover malformed diagnostics, missing values, pairing, correlation summaries, and hypothesis sections.
- Cargo workspace release tests, all Python tests, protocol tests, and `git diff --check` pass.
- One full external S6 run completes with a new output path and matching dataset digest.
- The report identifies PASS, FAIL, or UNAVAILABLE for every gate and records threats to validity.
- The result explicitly states whether a root cause is supported or unresolved and never claims automatic P0 qualification.
