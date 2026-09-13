# S5 Stability Remediation — 2026-09-13

## Scope

The remediation preserves protocol v1.1, five repetitions, 900 seconds total, concurrency 1, and exactly 100,000 measured frames per complete window. The published baseline remains immutable; no outliers were removed, no threshold was changed, and no repetition was selectively rerun.

## Artifacts

- Baseline raw: `D:\02-PERSONAL\TOOLS\prism-datasets\p0-100k-s5.jsonl` (SHA-256 `d192cf6d284990396dfcb6eb9b89b08d190b2b199129189c819819e984ba40fc`).
- Remediated raw: `D:\02-PERSONAL\TOOLS\prism-datasets\p0-100k-s5-remediated.jsonl` (SHA-256 `0dbda7ccab17142529b51e99fac7291ce394b93c8d4125bd783d1a87a66f5418`).
- Preflight: `D:\02-PERSONAL\TOOLS\prism-datasets\p0-100k-s5-remediated-preflight.json`.
- Remediated report: `docs/evidence/rust-s5-remediated-report.md`.
- Paired comparison: `docs/evidence/rust-s5-stability-comparison.md`.

## Results

The remediated package contains 615 valid windows. Correctness is PASS and RSS growth is PASS at 0.878967%. p99 stability remains FAIL: the maximum first-to-last change is 127.956989%, with failures in `b0:r1`, `b0:r3`, `b0:r4`, `b1:r1`, `b1:r3`, `b1:r4`, and `b2:r5`.

The new window timestamps are present and ordered. The comparison preserves common-window p99 values and reports unmatched tails; it does not smooth or discard observations. The evidence does not isolate a unique root cause. Resource growth is not supported as the primary cause; scheduler/frequency drift and orchestration effects remain unresolved hypotheses requiring controlled host telemetry.

## Decision

The diagnostic instrumentation and comparable rerun are complete, but the p99 gate is still failing. This is engineering evidence only and does not qualify S5 or PRISM for P0.
