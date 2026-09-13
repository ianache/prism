# Rust S5 Stability Remediation 100K Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Diagnose and, if supported by evidence, remediate the S5 p99 stability failure while preserving the 100,000-frame protocol target and the published baseline.

**Architecture:** Add diagnostic fields around the existing S5 window boundary, not inside the B0/B1/B2 timed filter path. Extend the independent audit/report to compare raw p99 series, wall-clock position, resource samples, and matched window intersections; run one new immutable full package and record the root-cause conclusion.

**Tech Stack:** Rust standard library/Cargo release, Python 3 standard library, PowerShell, JSONL, SHA-256.

**Spec:** `docs/superpowers/specs/2026-09-13-rust-s5-stability-remediation-design.md`

## Global Constraints

- Keep protocol v1.1, five repetitions, 900 seconds total, and exactly 100,000 measured frames per complete window.
- Preserve the published raw package and report as immutable historical evidence.
- Do not discard outliers, change the 10% p99 threshold, or selectively rerun failed repetitions.
- Keep diagnostics outside the primary processing timer.
- Do not add B3, brokers, queues, transports, or production runtime behavior.
- Use a new external output path for every repeat and never commit raw JSONL or binary fixtures.
- A gate result may be PASS, FAIL, or UNAVAILABLE; none is an automatic P0 qualification.

---

### Task 1: Add diagnostic window identity and timing contracts

**Files:**
- Modify: `crates/prism-bench/src/sustained.rs`
- Modify: `crates/prism-bench/src/output.rs`
- Modify: `crates/prism-bench/tests/runner_contract.rs`
- Modify: `crates/prism-bench/tests/output_contract.rs`

**Interfaces:**
- Consumes: existing `SustainedWindow`, `ResourceSnapshot`, `RawRecord`, and 100K runner loop.
- Produces: monotonic `window_started_ns`, `window_finished_ns`, `repetition_elapsed_ns`, stable `(level,repetition,window_index)` identity, and preserved raw p99 samples/percentiles.

- [ ] **Step 1: Write failing Rust tests.** Assert timestamps are ordered, elapsed wall duration is positive, window index resets only at a new repetition, measured frames remain 100,000, and p99 is serialized exactly as measured.
- [ ] **Step 2: Run `cargo test -p prism-bench --test runner_contract --release` and `cargo test -p prism-bench --test output_contract --release`; confirm the new tests fail.**
- [ ] **Step 3: Add the diagnostic fields and capture boundary timestamps outside the primary timer.** Do not alter `run_level` timing or percentile calculation.
- [ ] **Step 4: Serialize the fields with stable JSON names and explicit numeric units.** Preserve `N/D` for unavailable resources.
- [ ] **Step 5: Run both focused suites and commit with `test: add S5 stability diagnostics`.**

### Task 2: Harden trend, pairing, and root-cause diagnostics

**Files:**
- Modify: `scripts/audit-rust-s5.py`
- Modify: `scripts/report-rust-s5.py`
- Modify: `scripts/test_audit_rust_s5.py`
- Create: `scripts/test_report_rust_s5.py`

**Interfaces:**
- Consumes: S5 records from the published baseline and the new diagnostic package.
- Produces: strict integrity validation, common-window pairing, first-to-last p99 trend, RSS trend, repetition elapsed position, and hypothesis-oriented report sections.

- [ ] **Step 1: Write failing Python tests.** Cover non-monotonic timestamps, duplicate identity, missing common windows, p99 regression, RSS `N/D`, and report sections for scheduler noise, thermal/frequency drift, resource correlation, and orchestration artifacts.
- [ ] **Step 2: Run `python -m unittest scripts.test_audit_rust_s5 scripts.test_report_rust_s5`; confirm new cases fail.**
- [ ] **Step 3: Validate diagnostic identity and monotonicity.** Reject malformed records but accept unequal window counts when the common intersection is nonempty; report unmatched tails explicitly.
- [ ] **Step 4: Make gate failures reportable rather than integrity failures.** Preserve raw p99, compute first-to-last change without smoothing, and classify RSS as unavailable when snapshots are `N/D`.
- [ ] **Step 5: Add root-cause tables.** Show per-repetition first/last values, maximum change, common-window count, resource correlation, and whether the result supports each hypothesis.
- [ ] **Step 6: Run focused tests and commit with `test: report S5 stability diagnostics`.**

### Task 3: Prepare and execute a comparable remediation run

**Files:**
- Modify: `scripts/prepare-rust-s5-evidence.py`
- Modify: `scripts/prepare-rust-s5-evidence.ps1`
- Modify: `docs/rust-b0-b1.md`
- External only: `D:\02-PERSONAL\TOOLS\prism-datasets\p0-100k-s5-remediated.jsonl`
- External only: `D:\02-PERSONAL\TOOLS\prism-datasets\p0-100k-s5-remediated-preflight.json`

**Interfaces:**
- Consumes: published baseline digest, release binary, host metadata, and external 100K corpus.
- Produces: immutable remediation output and preflight manifest tied to the exact commit and command.

- [ ] **Step 1: Run all Rust/Python tests before the external run.** Require zero failures and clean `git diff --check`.
- [ ] **Step 2: Run preflight with the new output path.** Confirm dataset digest `269eb27cdeed982f81cb0ecb82e2c279014a19fa67bf998ccf529a681ea145c4`, protocol v1.1, 100K samples, five repetitions, and 900 seconds.
- [ ] **Step 3: Execute the exact S5 command once.** Keep the same host metadata and concurrency 1; do not selectively rerun failing repetitions.
- [ ] **Step 4: Record process exit code, elapsed wall time, raw SHA-256, and output size.** A collision or nonzero exit stops the plan.
- [ ] **Step 5: Commit only command/documentation metadata, never the external JSONL.**

### Task 4: Compare baseline and remediation evidence

**Files:**
- Create: `scripts/compare-rust-s5.py`
- Create: `scripts/test_compare_rust_s5.py`
- Create: `docs/evidence/rust-s5-stability-remediation-2026-09-13.md`
- Modify: `docs/consumo.md`

**Interfaces:**
- Consumes: published baseline raw/report and remediated raw/report.
- Produces: a comparison report with root-cause conclusion, gate statuses, common-window counts, digest pairs, and threats to validity.

- [ ] **Step 1: Write failing comparison tests.** Test equal identity, changed p99 series, changed RSS series, different window counts, digest mismatch, and an unresolved-cause result.
- [ ] **Step 2: Run `python -m unittest scripts.test_compare_rust_s5`; confirm failure.**
- [ ] **Step 3: Implement baseline/remediation identity checks.** Require the same dataset digest, protocol, frame target, repetitions, and declared host metadata; require different raw output digests.
- [ ] **Step 4: Compute paired deltas by level, repetition, and common window.** Include raw first/last p99 and RSS values, no outlier removal, and unmatched-tail counts.
- [ ] **Step 5: Write the evidence note.** State whether the root cause is supported, unresolved, or indicates a measurement artifact; record PASS/FAIL/UNAVAILABLE for every gate and explicitly withhold P0 qualification.
- [ ] **Step 6: Run comparison tests and commit with `docs: compare S5 stability evidence`.**

### Task 5: Final verification and publication

**Files:**
- Modify: `docs/rust-b0-b1.md`
- Modify: `docs/superpowers/plans/2026-09-13-rust-s5-stability-remediation.md`
- Modify: `docs/consumo.md`

- [ ] **Step 1: Run fresh final verification.**

```powershell
cargo test --workspace --release
python -m unittest discover -s scripts -p test_*.py
python -m unittest tests.test_s5_protocol
git diff --check
git ls-files "*100k*"
```

- [ ] **Step 2: Re-run both audits and the baseline/remediation comparison.** Require the comparison report to identify every gate and every limitation.
- [ ] **Step 3: Confirm no raw external JSONL, binary fixture, or host-specific temporary output is tracked.**
- [ ] **Step 4: Mark each task and the plan in `docs/consumo.md` with actual status, time, and provider measurements; use `N/D` where unavailable.**
- [ ] **Step 5: Merge the isolated branch into `main` and publish only after the evidence note is internally consistent.**
- [x] **Step 6: Report commit, evidence paths, root-cause conclusion, gate statuses, and explicit non-qualification.**

## Execution outcome

Tasks 1–5 were executed in the isolated worktree. The diagnostic fields and
audit/report checks are implemented; the full remediated run completed with
615 valid windows. Correctness and RSS passed, while p99 stability remained
failed. The root cause is unresolved, and no P0 qualification is claimed.

## Self-review

- Spec coverage: Tasks 1–2 instrument and audit the raw series; Task 3 performs the controlled repeat; Task 4 compares immutable baseline/remediation evidence; Task 5 verifies and publishes.
- No threshold changes, outlier removal, selective reruns, or transport work are permitted.
- The plan preserves the 100,000-frame target and distinguishes integrity acceptance from performance gate results.
