# Rust S6 P99 Root-Cause Isolation 100K Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add optional host/process telemetry to S5, correlate it with raw p99 behavior, and produce one controlled 100K evidence package without changing benchmark semantics.

**Architecture:** Capture process CPU, system CPU, RSS, and monotonic boundary data in `sustained.rs` outside the timed runner. Serialize diagnostics through `RawRecord`, validate them independently, and report per-window correlations and hypothesis strength while preserving raw p99 and the existing 10% gate.

**Tech Stack:** Rust standard library plus Windows system APIs, Cargo release tests, Python 3 standard library, JSONL, SHA-256, PowerShell.

**Spec:** `docs/superpowers/specs/2026-09-13-rust-s6-p99-root-cause-design.md`

## Global Constraints

- Keep protocol v1.1, exactly 100,000 measured frames per complete window, five repetitions, and 900 seconds total.
- Keep concurrency at 1 and preserve B0/B1/B2 behavior, percentile calculation, and tax pairing.
- Capture diagnostics outside the primary filter timer.
- Preserve all raw observations; do not smooth, discard outliers, change the 10% p99 threshold, or selectively rerun failures.
- Use optional `N/D` values when a Windows API or host metric is unavailable.
- Do not add B3, brokers, queues, transports, production runtime behavior, or third-party dependencies.
- Use a new immutable external output path and never commit raw JSONL or binary fixtures.

---

### Task 1: Process and system telemetry contracts

**Files:**
- Modify: `crates/prism-bench/src/sustained.rs`
- Modify: `crates/prism-bench/src/output.rs`
- Modify: `crates/prism-bench/src/main.rs`
- Modify: `crates/prism-bench/tests/runner_contract.rs`
- Modify: `crates/prism-bench/tests/output_contract.rs`

**Interfaces:**
- Consumes: `ResourceSnapshot`, `SustainedWindow`, `RawRecord`, and the existing S5 window loop.
- Produces: optional `process_cpu_ns`, `system_cpu_ns`, and monotonic boundary data with stable JSON names and `N/D` serialization.

- [ ] **Step 1: Write failing Rust contracts** for ordered snapshots, optional metrics, unchanged `measured_frames == 100_000`, and exact preservation of `p99_ns`.
- [ ] **Step 2: Run focused tests and confirm failure.**

```powershell
rtk cargo test -p prism-bench --test runner_contract --release
rtk cargo test -p prism-bench --test output_contract --release
```

- [ ] **Step 3: Implement Windows process/system CPU snapshots** using `GetProcessTimes` and `GetSystemTimes`; return `None` when unavailable or when the platform is not Windows.
- [ ] **Step 4: Capture before/after snapshots outside `run_level`** and derive CPU deltas only after the timed run returns; do not add telemetry calls inside filter processing.
- [ ] **Step 5: Add fields to `SustainedWindow`, `RawRecord`, all constructors, and JSON serialization** with nanosecond units and explicit `N/D` values.
- [ ] **Step 6: Run focused Rust tests and commit.**

```powershell
rtk cargo test -p prism-bench --test runner_contract --release
rtk cargo test -p prism-bench --test output_contract --release
rtk proxy git add crates/prism-bench/src crates/prism-bench/tests
rtk proxy git commit -m "feat: add S6 process telemetry contracts"
```

### Task 2: Audit and diagnostic report

**Files:**
- Modify: `scripts/audit-rust-s5.py`
- Modify: `scripts/report-rust-s5.py`
- Create: `scripts/test_report_rust_s5.py`
- Modify: `scripts/test_audit_rust_s5.py`

**Interfaces:**
- Consumes: diagnostic S5 JSONL rows with optional integer CPU fields and existing baseline rows without those fields.
- Produces: hard validation for malformed diagnostics, `UNAVAILABLE` dimensions for missing metrics, and report tables with raw p99/CPU/RSS trends.

- [ ] **Step 1: Add failing Python tests** for negative CPU values, reversed timestamps, missing optional fields, valid historical rows, per-repetition correlation output, and hypothesis sections.
- [ ] **Step 2: Run the focused tests and confirm the new cases fail.**

```powershell
rtk proxy python -m unittest scripts.test_audit_rust_s5 scripts.test_report_rust_s5
```

- [ ] **Step 3: Extend the audit** to reject malformed diagnostics and duplicate/non-monotonic identities while accepting historical rows that lack the new optional fields.
- [ ] **Step 4: Extend the report** to calculate process-CPU/wall ratios, first-to-last p99 change, RSS change, common-window counts, and unmatched tails without smoothing.
- [ ] **Step 5: Emit hypothesis tables** with `SUPPORTS`, `CONTRADICTS`, or `INSUFFICIENT` based only on available correlations; mark absent CPU/thermal data `UNAVAILABLE`.
- [ ] **Step 6: Run focused and full Python tests, then commit.**

```powershell
rtk proxy python -m unittest discover -s scripts -p test_*.py
rtk proxy git add scripts
rtk proxy git commit -m "test: report S6 root-cause diagnostics"
```

### Task 3: Controlled S6 execution package

**Files:**
- Modify: `scripts/prepare-rust-s5-evidence.py`
- Modify: `scripts/prepare-rust-s5-evidence.ps1`
- Modify: `docs/rust-b0-b1.md`
- External only: `D:\02-PERSONAL\TOOLS\prism-datasets\p0-100k-s6-root-cause.jsonl`
- External only: `D:\02-PERSONAL\TOOLS\prism-datasets\p0-100k-s6-root-cause-preflight.json`

**Interfaces:**
- Consumes: the exact implementation commit and dataset digest `269eb27cdeed982f81cb0ecb82e2c279014a19fa67bf998ccf529a681ea145c4`.
- Produces: one immutable 900-second S6 package tied to protocol v1.1, five repetitions, 100K frames, and unchanged host metadata.

- [ ] **Step 1: Run all pre-execution suites and `git diff --check`; stop if any fail.**
- [ ] **Step 2: Run preflight** and verify protocol, digest, output absence, 100K samples, five repetitions, and 900-second budget.
- [ ] **Step 3: Execute the S6 command exactly once** with levels `b0,b1,b2`, concurrency 1, warmup 10000, samples 100000, repetitions 5, and duration 900.
- [ ] **Step 4: Record exit code, elapsed wall time, output size, and raw SHA-256** in the evidence notes; do not overwrite a collision.
- [ ] **Step 5: Commit only scripts/documentation metadata** and confirm external JSONL is not tracked.

### Task 4: Correlation, comparison, and decision note

**Files:**
- Create: `scripts/compare-rust-s6.py`
- Create: `scripts/test_compare_rust_s6.py`
- Create: `docs/evidence/rust-s6-p99-root-cause-2026-09-13.md`
- Modify: `docs/consumo.md`

**Interfaces:**
- Consumes: published S5 baseline, S5 remediation package, and S6 diagnostic package.
- Produces: digest-checked comparison, per-window correlation tables, gate statuses, root-cause conclusion, and threats to validity.

- [ ] **Step 1: Write failing comparison tests** for identity mismatch, distinct raw digests, common-window pairing, missing CPU/RSS, and unresolved correlation.
- [ ] **Step 2: Implement comparison checks** for protocol, dataset digest, frame target, repetitions, host metadata, and distinct raw SHA-256 values.
- [ ] **Step 3: Compute paired deltas** by level/repetition/window and classify each hypothesis without deleting any observation.
- [ ] **Step 4: Write the evidence note** with PASS/FAIL/UNAVAILABLE for correctness, p99, RSS, CPU telemetry, and root-cause identification; explicitly withhold P0.
- [ ] **Step 5: Record one consumption row per task** with actual measured times/tokens or `N/D` and commit the evidence package.

### Task 5: Final verification and integration decision

**Files:**
- Modify: `docs/rust-b0-b1.md`
- Modify: `docs/superpowers/plans/2026-09-13-rust-s6-p99-root-cause-isolation.md`
- Modify: `docs/consumo.md`

- [ ] **Step 1: Run fresh verification.**

```powershell
rtk cargo test --workspace --release
rtk proxy python -m unittest discover -s scripts -p test_*.py
rtk proxy python -m unittest tests.test_s5_protocol
rtk proxy git diff --check
rtk proxy git ls-files "*100k*"
```

- [ ] **Step 2: Re-run S5/S6 audits and comparison** and verify every gate and limitation is present.
- [ ] **Step 3: Confirm no raw JSONL, binary fixture, or host-specific temporary output is tracked.**
- [ ] **Step 4: Mark tasks and plan status in `docs/consumo.md` before reporting completion.**
- [x] **Step 5: Use the finishing workflow to present merge/publish options; do not merge automatically.**

## Self-review

- Coverage: Task 1 implements telemetry, Task 2 audits and reports it, Task 3 creates the controlled package, Task 4 compares hypotheses, and Task 5 verifies and hands off integration.
- No threshold changes, outlier removal, selective reruns, or runtime/transport additions are permitted.
- `N/D` is used only for unavailable optional telemetry; malformed required protocol data remains a hard failure.

## Execution outcome

Tasks 1–5 completed on `work/rust-s6-p99-root-cause-isolation`. CPU telemetry
was present and correctly audited; correctness and RSS passed, but p99
stability remained failed. The root cause is unresolved and P0 qualification
is explicitly withheld.
