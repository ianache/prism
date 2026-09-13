# S6 P99 Root-Cause Isolation — 2026-09-13

## Scope

S6 preserves protocol v1.1, concurrency 1, five repetitions, a 900-second
total budget, and exactly 100,000 measured frames per complete window. The S5
baseline and remediation packages remain immutable. No outlier, threshold, or
selective rerun policy was changed.

## Artifacts

- Dataset digest: `269eb27cdeed982f81cb0ecb82e2c279014a19fa67bf998ccf529a681ea145c4`.
- S6 raw: `D:\02-PERSONAL\TOOLS\prism-datasets\p0-100k-s6-root-cause.jsonl` (SHA-256 `e7c8be5561f773d474b7dc0938e57c88868792226ffb1254d4c80798c3e08f7e`).
- S6 preflight: `D:\02-PERSONAL\TOOLS\prism-datasets\p0-100k-s6-root-cause-preflight.json`.
- S6 audit: `D:\02-PERSONAL\TOOLS\prism-datasets\p0-100k-s6-root-cause-audit.txt`.
- S6 report: `docs/evidence/rust-s6-root-cause-report.md`.
- Three-package comparison: `docs/evidence/rust-s6-p99-root-cause-comparison.md`.

## Results

The package contains 549 valid windows. Timestamp ordering and CPU telemetry
are present. Correctness is PASS. RSS growth is PASS at 0.882947%. p99
stability remains FAIL at the provisional 10% threshold, with failures in
`b0:r1`, `b0:r3`, `b0:r4`, `b1:r2`, `b1:r5`, and `b2:r1`; maximum first-to-last
change is 480.952381%.

Process CPU/wall ratios are available per window, but the single-host run does
not provide independent frequency or thermal telemetry. Bounded RSS does not
support resource growth as the primary cause. The observed data is therefore
consistent with scheduler noise or frequency/thermal drift, but cannot
distinguish them. Orchestration remains a possible contributor because window
counts differ by repetition, while common-window pairing remains intact.

## Decision

S6 instrumentation, controlled execution, comparison, and audit are complete.
The root cause remains unresolved and the p99 gate still fails. This is
engineering evidence only; no S5/S6 or PRISM P0 qualification is claimed.
