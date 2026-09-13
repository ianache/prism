# Rust S5 Sustained Load Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add an auditable Rust S5 sustained-load scenario for B0, B1, and B2 using five repetitions and fixed 100,000-frame comparison windows over a 900-second total budget.

**Architecture:** Extend the existing `prism-bench` CLI and runner with a windowed sustained-load orchestrator. Each level runs five sequential 180-second repetitions, processing complete 100,000-frame windows and sampling process resources outside the primary timer. A dedicated Python audit validates the variable-length window series and computes stability, RSS growth, and matched per-window taxes without adding a queue, broker, or transport.

**Tech Stack:** Rust stable/Cargo standard library, Python 3 standard library, canonical JSONL, Windows process APIs, existing deterministic corpus and oracle.

**Spec:** `docs/superpowers/specs/2026-09-13-rust-s5-sustained-load-design.md`

## Global Constraints

- Use protocol v1.1 and exactly 100,000 measured frames in every complete decision window.
- Use five repetitions for every level and divide a 900-second total budget into 180 seconds per repetition.
- Preserve B0/B1/B2 behavior, F1-F6 order, S1-S4 CLI behavior, and existing taxes.
- Use the resident external 100K corpus; do not add binary fixtures to Git.
- Keep I/O, scheduling, sampling, aggregation, serialization, and writing outside the primary processing timer.
- Use `N/D` for unavailable optional resource metrics; never synthesize values.
- Raw output is write-once and a partial final window is reported but excluded from comparable records.
- No B3, queue, broker, network transport, persistent backpressure, multi-hour soak, or P0 qualification claim.

---

### Task 1: Add the S5 CLI and protocol contract

**Files:**
- Modify: `crates/prism-bench/src/cli.rs`
- Modify: `crates/prism-bench/tests/cli_contract.rs`
- Create: `tests/test_s5_protocol.py`
- Modify: `docs/rust-b0-b1.md`

**Interfaces:**
- Consumes: existing `Config` parsing for `--scenario`, `--levels`, `--samples`, `--repetitions`, and `--output`.
- Produces: `Config.duration_seconds: u64` with default `900` for S5; S5 validation requiring `samples == 100_000`, `repetitions == 5`, `concurrency == 1`, and a positive duration divisible by five; rejection of S5-only flags for S1-S4.

- [ ] **Step 1: Write the failing Rust CLI tests.** Add cases for accepting S5 with `--samples 100000 --repetitions 5 --duration-seconds 900`, defaulting the duration when omitted, and rejecting zero duration, non-100K samples, non-five repetitions, and concurrency values other than one.

- [ ] **Step 2: Run the focused tests and verify they fail.**

Run: `cargo test -p prism-bench --test cli_contract --release`

Expected: the new S5 cases fail because the scenario and duration field are not implemented.

- [ ] **Step 3: Implement minimal CLI parsing and validation.** Add `--duration-seconds` to the accepted options, parse it as `u64`, retain it in `Config`, and apply the S5 constraints after the existing common validation. Keep the existing S1, S2, S3, and S4 branches unchanged.

- [ ] **Step 4: Add the Python protocol assertions.** In `tests/test_s5_protocol.py`, assert that the frozen protocol documents 100,000-frame decisions, five repetitions, and the 15-minute S5 duration; assert that the audit script exposes the S5 scenario name and window fields.

- [ ] **Step 5: Update the operator documentation.** Add the exact PowerShell-compatible S5 command shape, the 900-second default, the five 180-second repetitions, the immutable output rule, and the audit command to `docs/rust-b0-b1.md`. State that S5 evidence is not P0 qualification.

- [ ] **Step 6: Run focused verification and commit.**

Run: `cargo test -p prism-bench --test cli_contract --release`

Run: `python -m unittest tests.test_s5_protocol`

Commit: `git add crates/prism-bench/src/cli.rs crates/prism-bench/tests/cli_contract.rs tests/test_s5_protocol.py docs/rust-b0-b1.md && git commit -m "feat: add S5 sustained-load CLI contract"`

### Task 2: Implement fixed-window sustained execution and resource sampling

**Files:**
- Create: `crates/prism-bench/src/sustained.rs`
- Modify: `crates/prism-bench/src/lib.rs`
- Modify: `crates/prism-bench/src/runner.rs`
- Modify: `crates/prism-bench/tests/runner_contract.rs`

**Interfaces:**
- Consumes: `Dataset`, `RunConfig`, `Level`, `run_level`, percentile helpers, and the existing B2 collector.
- Produces: `SustainedConfig { duration_seconds: u64, window_frames: usize, repetitions: usize }`, `ResourceSnapshot { rss_bytes: Option<u64>, sampled_at_ns: u128 }`, `SustainedWindow`, and `run_sustained(level, dataset, config) -> Result<SustainedRun, RunError>`.

- [ ] **Step 1: Write failing runner tests.** Add tests for exactly 100,000 frames in a complete window, five repetition identities, monotonic window indices, deterministic corpus wraparound, partial-tail exclusion, and explicit `None` for unavailable resource values.

- [ ] **Step 2: Run the focused runner tests and verify failure.**

Run: `cargo test -p prism-bench --test runner_contract --release`

Expected: the S5 types and runner function are missing and the new tests fail to compile.

- [ ] **Step 3: Define the sustained-run data types.** Store level, repetition, window index, measured frame count, primary elapsed nanoseconds, latency samples/percentiles, throughput, correctness counters, resource snapshots, and an incomplete-tail count. Keep each complete window independent so the audit can compare first and last windows.

- [ ] **Step 4: Implement the fixed-window loop.** For each level and repetition, run warm-up once before the measured budget, process resident frames in deterministic order, stop only after the repetition deadline, emit complete 100K windows, and discard the incomplete tail while retaining its count. Use the existing B0/B1/B2 processing functions and correctness checks.

- [ ] **Step 5: Implement resource sampling outside the timer.** On Windows, read the current process working-set/RSS through a standard-library-compatible system query or a narrowly scoped platform helper. Return `None` when a metric cannot be read. Capture timestamps and snapshots immediately before and after each window without including sampling time in frame latency or throughput.

- [ ] **Step 6: Run focused tests and commit.**

Run: `cargo test -p prism-bench --test runner_contract --release`

Commit: `git add crates/prism-bench/src/sustained.rs crates/prism-bench/src/lib.rs crates/prism-bench/src/runner.rs crates/prism-bench/tests/runner_contract.rs && git commit -m "feat: run fixed-window S5 sustained load"`

### Task 3: Serialize S5 windows and compute matched taxes

**Files:**
- Modify: `crates/prism-bench/src/output.rs`
- Modify: `crates/prism-bench/src/main.rs`
- Modify: `crates/prism-bench/tests/output_contract.rs`
- Modify: `crates/prism-bench/tests/bench_contract.rs`

**Interfaces:**
- Consumes: `SustainedRun`, `SustainedWindow`, existing `RawRecord`, and `write_once_atomic`.
- Produces: `RawRecord` fields `window_index`, `duration_seconds`, `rss_before_bytes`, `rss_after_bytes`, `incomplete_tail_frames`, and S5 identity; taxes paired by `(level, repetition, window_index)`.

- [ ] **Step 1: Write failing output tests.** Assert JSON serialization of S5 identity, 100K measured frames, window index, duration, resource values including `N/D`, incomplete-tail count, first/last window metrics, and B2 F1-F6 evidence. Add a tax test proving B1 matches B0 and B2 matches B1 for the same repetition and window.

- [ ] **Step 2: Run focused output tests and verify failure.**

Run: `cargo test -p prism-bench --test output_contract --release`

Expected: the new S5 fields and keyed tax helper are absent.

- [ ] **Step 3: Extend `RawRecord` and serialization.** Add typed S5 fields with stable JSON names. Serialize unavailable resource values as `"N/D"` and preserve numeric zeroes. Keep S1-S4 records backward-compatible by using their existing fields and scenario-specific construction.

- [ ] **Step 4: Implement keyed tax pairing.** Build lookup keys from level, repetition, and window index; calculate workflow tax from B0/B1 p99 and observability tax from B1/B2 p99; retain negative values and reject duplicate or missing comparison keys before writing.

- [ ] **Step 5: Wire `run_s5` into the CLI.** Add a scenario branch in `main.rs`, execute levels in b0/b1/b2 order, flatten complete windows into JSONL records, attach host/dataset metadata, and use the existing atomic write-once function. Do not alter S1-S4 orchestration.

- [ ] **Step 6: Run focused tests and commit.**

Run: `cargo test -p prism-bench --test output_contract --release`

Run: `cargo test -p prism-bench --test bench_contract --release`

Commit: `git add crates/prism-bench/src/output.rs crates/prism-bench/src/main.rs crates/prism-bench/tests/output_contract.rs crates/prism-bench/tests/bench_contract.rs && git commit -m "feat: serialize S5 windows and matched taxes"`

### Task 4: Build the independent S5 audit

**Files:**
- Create: `scripts/audit-rust-s5.py`
- Create: `scripts/test_audit_rust_s5.py`
- Modify: `scripts/test_audit_rust_s4.py` only if shared helpers are extracted without changing S4 behavior

**Interfaces:**
- Consumes: `--dataset PATH --raw PATH`, the dataset manifest/digest, and S5 JSONL records.
- Produces: success output `ok levels=b0,b1,b2 repetitions=5 frames=100000`, gate diagnostics for p99/RSS trends, and nonzero exit status for invalid evidence.

- [ ] **Step 1: Write failing audit tests.** Create valid synthetic multi-window records and invalid cases for missing level/repetition, duplicate window, non-100K frame count, incomplete window emitted as comparable, non-finite metric, dataset mismatch, correctness mismatch, missing resource representation, and unpaired tax.

- [ ] **Step 2: Run the audit tests and verify failure.**

Run: `python -m unittest scripts.test_audit_rust_s5`

Expected: the audit module is missing.

- [ ] **Step 3: Implement strict identity and cardinality checks.** Require scenario S5, protocol v1.1, concurrency 1, all three levels, repetitions 1 through 5, at least one complete window per tuple, strictly increasing window indices, exactly 100,000 measured frames, and matching dataset identity.

- [ ] **Step 4: Implement metric and gate checks.** Validate finite latency/throughput numbers, 100% correctness, explicit `N/D` handling, incomplete-tail accounting, first-to-last p99 change, RSS growth when both snapshots are available, and per-window workflow/observability tax keys.

- [ ] **Step 5: Implement concise audit output and run focused tests.** Print the valid summary plus gate values; report the first failing invariant for malformed input.

Run: `python -m unittest scripts.test_audit_rust_s5 scripts.test_audit_rust_s4`

Commit: `git add scripts/audit-rust-s5.py scripts/test_audit_rust_s5.py && git commit -m "test: audit S5 sustained-load evidence"`

### Task 5: Execute, verify, document, and publish the S5 package

**Files:**
- Modify: `docs/rust-b0-b1.md`
- Modify: `docs/superpowers/plans/2026-09-13-rust-s5-sustained-load.md`
- Modify: `docs/consumo.md`
- External only: `D:\02-PERSONAL\TOOLS\prism-datasets\p0-100k-s5.jsonl`

**Interfaces:**
- Consumes: the completed S5 CLI, runner, serializer, audit, external dataset, and host metadata.
- Produces: immutable external S5 JSONL, audit output, documented digest and gate results, clean Git verification, and a published merge on `main`.

- [ ] **Step 1: Run preflight verification.**

Run: `cargo test --workspace --release`

Run: `python -m unittest discover -s scripts -p "test_*.py"`

Run: `python -m unittest tests.test_s5_protocol`

Run: `rtk git diff --check`

Expected: all tests pass and no whitespace errors are reported.

- [ ] **Step 2: Execute the external run once.** Refuse to overwrite an existing output path. Use the documented S5 command with `--samples 100000`, `--repetitions 5`, `--duration-seconds 900`, explicit host metadata, and the external 100K dataset.

- [ ] **Step 3: Audit and parse every output line.** Run `python scripts/audit-rust-s5.py --dataset D:\02-PERSONAL\TOOLS\prism-datasets\p0-100k --raw D:\02-PERSONAL\TOOLS\prism-datasets\p0-100k-s5.jsonl`, parse each non-empty line with `json.loads`, verify unique keys, and record the dataset digest and number of complete windows.

- [ ] **Step 4: Document evidence and limitations.** Record duration, host metadata, digest, window counts, incomplete tails, p99 trend, RSS availability/growth, tax ranges, and explicit non-qualification. Mark every completed task and add one task row plus one plan row to `docs/consumo.md` with N/D for unavailable provider token/time measurements.

- [ ] **Step 5: Run final verification.**

Run: `cargo test --workspace --release`

Run: `python -m unittest discover -s scripts -p "test_*.py"`

Run: `rtk git diff --check`

Run: `rtk git ls-files "*100k*"`

Expected: all suites pass, no binary S5 fixture is tracked, and the only intentional untracked data is outside the repository.

- [ ] **Step 6: Commit, merge, and publish.** Commit the documentation and consumption record, merge the isolated worktree branch into `main` only after verification is green, push `main` to `origin`, and report the commit, evidence path, audit summary, and any gate failures without claiming P0 qualification.

## Self-review

- Spec coverage: Tasks 1-3 cover CLI, fixed windows, five repetitions, resource sampling, serialization, and keyed taxes; Task 4 covers every malformed-evidence invariant; Task 5 covers external evidence and publication.
- Placeholder scan: no `TBD`, `TODO`, or unspecified implementation step remains; all external command paths and required parameters are concrete.
- Type consistency: `SustainedConfig`, `ResourceSnapshot`, `SustainedWindow`, and `SustainedRun` are introduced in Task 2 and consumed by Tasks 3 and 5; `RawRecord` fields are introduced in Task 3 and consumed by the audit in Task 4.
- Scope check: B3, broker, transport, persistent queue, multi-hour soak, and P0 selection remain explicitly excluded.
