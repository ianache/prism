# Rust S5 Stability Remediation 100K Design

**Status:** Proposed for review  
**Date:** 2026-09-13  
**Scope:** Diagnosis and controlled remediation of the S5 p99 stability gate.

## Goal

Explain and, where evidence supports it, remediate the S5 p99 degradation
observed in the external 100K run. Produce a repeatable stability package
without changing the 100,000-frame decision target, the provisional 10% gate,
or the B0/B1/B2 processing semantics.

## Observed baseline

The published S5 package contains 515 complete windows. Correctness passed and
maximum RSS growth was 0.669886%, but the p99 stability gate failed for
`b1:r2`, `b1:r3`, `b1:r4`, and `b2:r5`; the maximum first-to-last increase was
124.015748%. This result is a valid negative gate result, not a P0 decision.

## Constraints

- Keep protocol v1.1 and exactly 100,000 measured frames per comparable
  window.
- Keep five repetitions and the 900-second total S5 budget.
- Do not change the p99 threshold, discard outliers, or selectively rerun only
  failing repetitions.
- Do not add brokers, transports, queues, B3, or production runtime behavior.
- Preserve the existing raw package and report as immutable historical evidence.
- Any remediation must be isolated, documented, and compared against the
  published baseline using a new output path.

## Diagnostic model

The next run records enough context to distinguish four hypotheses:

1. scheduler/OS noise causes isolated window spikes;
2. CPU frequency or thermal drift causes monotonic slowdown;
3. resource growth or allocation behavior correlates with p99 growth;
4. the window orchestration or tax pairing creates a measurement artifact.

For every window, retain the existing latency and correctness metrics plus
monotonic start/end timestamps, elapsed wall time, processed frame count,
throughput, RSS before/after, repetition budget position, and a stable window
key. The diagnostic report must show both the raw p99 series and a derived
first-to-last trend; it must not replace the gate metric with a smoothed value.

## Remediation boundaries

Only changes that preserve the timed B0/B1/B2 behavior are allowed. Examples
include correcting window scheduling/accounting, moving orchestration outside
the primary timer, fixing an evidence-key mismatch, or adding host metadata.
Changing filter logic, removing valid samples, increasing the threshold, or
adding runtime dependencies is out of scope.

The remediation is accepted only if a new full S5 run is comparable with the
published baseline and the report explicitly states whether the p99 gate is
PASS, FAIL, or UNAVAILABLE. A PASS does not imply P0 qualification.

## Verification and acceptance

- Existing Cargo and Python suites remain green.
- New diagnostic contracts cover timestamp monotonicity, 100K windows,
  repetition/window identity, raw p99 preservation, and no outlier removal.
- The baseline package remains auditable and unchanged.
- A new full external run uses a new immutable output path and the same dataset
  digest.
- The report explains the root cause or explicitly records that it remains
  unresolved.
- Any gate result is published with threats to validity and without a P0
  qualification claim.
