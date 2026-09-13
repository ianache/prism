# Rust S4 Burst Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add an auditable Rust S4 burst benchmark for B0, B1, and B2 across baseline, burst, and recovery phases using 100,000 frames per phase.

**Architecture:** Extend the existing Rust benchmark with a synchronous phase scheduler. A preflight B0 calibration at concurrency 1 selects a baseline offered rate at 80% of median capacity; each repetition then processes baseline, 10× burst, and recovery phases over resident frames. JSONL output and a Python audit preserve phase/repetition pairing without introducing a queue or broker.

**Tech Stack:** Rust stable/Cargo standard library, Python 3 standard library, canonical JSONL, Windows MSVC release toolchain.

**Spec:** `docs/superpowers/specs/2026-09-13-rust-s4-burst-design.md`

## Global Constraints

- Use protocol v1.1 and exactly 100,000 frames in each phase.
- Use exactly five measured repetitions for each level and phase.
- Use phases `baseline`, `burst`, and `recovery` in that exact order.
- Produce exactly 45 records: 3 levels × 3 phases × 5 repetitions.
- Calibrate with B0, concurrency 1, 100,000 frames, five repetitions; baseline rate is 80% of median calibration throughput and burst rate is 10× baseline.
- Use the same resident external 100K corpus and host metadata for every phase and level.
- Keep schedule creation, dataset I/O, correctness comparison, aggregation, serialization, and writing outside the primary processing timer.
- Measure per-frame latency and phase-group processing throughput separately from offered arrival rate.
- Pair workflow and observability taxes by identical phase and repetition; preserve negative values.
- Keep binary fixtures outside Git and raw output write-once.
- Do not implement a queue, broker, transport, S5 sustained load, cross-language runners, or P0 qualification.

---

### Task 1: Define the S4 CLI and phase contract

**Files:**
- Modify: `crates/prism-bench/src/cli.rs`
- Modify: `crates/prism-bench/src/main.rs`
- Modify: `crates/prism-bench/tests/cli_contract.rs`
- Create: `tests/test_s4_protocol.py`
- Modify: `docs/rust-b0-b1.md`

**Interfaces:**
- `Config` accepts `scenario: "S4"` and retains one S3-compatible `concurrency`, defaulting to 1 for S4.
- S4 requires `b0,b1,b2`, `samples=100000`, `repetitions=5`, and `concurrency=1` for the first package.
- `Phase` is an internal enum with variants `Baseline`, `Burst`, and `Recovery`, serialized as the exact lowercase names.
- `main.rs` creates 45 records from the three levels, three phases, and five repetitions.

- [x] Add failing Rust/Python tests for S4 acceptance, rejection of missing B0/B1/B2, rejection of non-1 concurrency, rejection of wrong frame target, exact phase names/order, and expected 45-record cardinality.
- [x] Run `python -m unittest tests.test_s4_protocol` and `cargo test -p prism-bench --test cli_contract --release`; confirm the S4 tests fail before implementation.
- [x] Implement S4 validation while preserving S1, S2, and S3 CLI behavior.
- [x] Add deterministic IDs containing scenario, level, phase, and repetition, such as `S4-b2-burst-r4`.
- [x] Document the exact S4 command and phase semantics in `docs/rust-b0-b1.md`.
- [x] Run focused protocol and CLI suites and confirm all pass.
- [x] Append the task consumption row and commit with `feat: add S4 burst CLI contract`.

### Task 2: Implement calibration, schedule, and phase execution

**Files:**
- Modify: `crates/prism-bench/src/runner.rs`
- Modify: `crates/prism-bench/tests/runner_contract.rs`
- Create: `crates/prism-bench/src/burst.rs`
- Modify: `crates/prism-bench/src/lib.rs`

**Interfaces:**
- `Calibration { throughputs: Vec<f64>, median_frames_per_sec: f64, baseline_frames_per_sec: f64, burst_frames_per_sec: f64, baseline_interval_ns: u128, burst_interval_ns: u128 }` stores the five B0 calibration results and derived rates.
- `PhaseSchedule::new(calibration: &Calibration, phase: Phase, frames: usize) -> PhaseSchedule` creates deterministic arrival offsets before timing.
- `run_phase(level: Level, dataset: &Dataset, config: &RunConfig, schedule: &PhaseSchedule) -> Result<PhaseRun, RunError>` processes one phase and returns samples, counters, lateness metrics, and B2 evidence.
- `run_level` retains its existing API for S1/S2/S3; S4 orchestration may use new phase-specific helpers.

- [x] Add failing tests for median calibration, exact 80% baseline rate, exact 10× burst rate, interval derivation, deterministic phase timestamps, and 100,000 scheduled frames.
- [x] Add failing runner tests for ordered phases, complete frame coverage, on-time versus late accounting, correctness preservation, and B2 collector ownership.
- [x] Run the focused burst and runner tests and confirm expected failures.
- [x] Implement `burst.rs` using integer nanosecond intervals and deterministic cumulative offsets; reject zero/non-finite rates and arithmetic overflow.
- [x] Implement B0 calibration with five measured repetitions at concurrency 1 before S4 phases; keep calibration metadata outside the phase timer.
- [x] Implement synchronous phase processing over resident bytes. Record each frame's scheduled offset and processing-start offset, but do not sleep inside the primary processing timer.
- [x] Preserve existing warm-up, nearest-rank percentiles, correctness aborts, typed rejection counts, and B2 F1–F6 evidence.
- [x] Run runner contracts plus all existing B0/B1/B2 runtime contracts.
- [x] Append the task consumption row and commit with `feat: run calibrated S4 burst phases`.

### Task 3: Extend raw output and pair phase taxes

**Files:**
- Modify: `crates/prism-bench/src/output.rs`
- Modify: `crates/prism-bench/src/main.rs`
- Modify: `crates/prism-bench/tests/output_contract.rs`
- Modify: `crates/prism-bench/tests/bench_contract.rs`

**Interfaces:**
- `RawRecord` gains `phase`, `offered_frames_per_sec`, `processed_frames_per_sec`, `late_frames`, `on_time_frames`, `lateness_p50_ns`, `lateness_p95_ns`, `lateness_p99_ns`, `calibration_median_frames_per_sec`, `baseline_frames_per_sec`, and `burst_frames_per_sec`.
- Tax baseline maps use `BTreeMap<(Phase, usize), u128>` keyed by phase and repetition.
- S4 workflow tax is B1 p99 versus B0 p99 for the same phase/repetition; observability tax is B2 p99 versus B1 p99 for the same phase/repetition.
- JSON serialization retains deterministic key ordering and writes 45 records atomically.

- [x] Add failing tests that parse JSON and assert all S4 identity, phase, offered-rate, processed-rate, lateness, calibration, latency, throughput, correctness, and B2 fields.
- [x] Add failing tax tests proving phase/repetition matching and preservation of negative values.
- [x] Run focused output/benchmark contracts and confirm expected failures.
- [x] Implement new fields and phase-keyed tax lookup without changing S1/S2/S3 schemas beyond additive compatibility.
- [x] Build one immutable record per `(level, phase, repetition)` and reject duplicate output paths.
- [x] Run output contracts, benchmark contracts, and the complete Rust workspace suite.
- [x] Append the task consumption row and commit with `feat: serialize S4 phase metrics`.

### Task 4: Build the independent S4 audit

**Files:**
- Create: `scripts/audit-rust-s4.py`
- Create: `scripts/test_audit_rust_s4.py`
- Modify: `docs/rust-b0-b1.md`

**Interfaces:**
- `audit-rust-s4.py --dataset <path> --raw <jsonl>` requires exactly 45 records and prints `ok rows=45 levels=b0,b1,b2 phases=baseline,burst,recovery frames=100000` on valid evidence.
- The audit requires one record for every level, phase, and repetition tuple.
- It validates protocol identity, phase order, frame count, dataset digest/ID, finite metrics, correctness, lateness counters, calibration relationships, and phase-keyed taxes.

- [x] Create valid temporary fixtures and invalid cases for missing phase, duplicate tuple, wrong phase order, wrong scenario, wrong frame count, digest mismatch, correctness mismatch, non-finite metric, negative count, invalid calibration ratio, wrong workflow tax, and wrong observability tax.
- [x] Run `python -m unittest scripts.test_audit_rust_s4` and confirm the new tests fail before implementation.
- [x] Implement standard-library JSONL parsing, exact tuple validation, calibration/rate invariants, and phase/repetition p99 tax arithmetic.
- [x] Run S4 audit tests with all existing S1, S2, and S3 audit suites.
- [x] Document the audit command, append the task consumption row, and commit with `test: audit S4 burst evidence`.

### Task 5: Execute, verify, and publish the S4 evidence package

**Files:**
- Modify: `results/reports/README.md`
- Modify: `results/decision-matrix/README.md`
- Modify: `results/raw/README.md`
- Modify: `docs/consumo.md`
- Modify: `docs/superpowers/plans/2026-09-13-rust-s4-burst.md`

**Interfaces:**
- External output: `D:\02-PERSONAL\TOOLS\prism-datasets\p0-100k-s4.jsonl`; if it exists, stop and choose a new explicitly named immutable path.
- Exact command uses `--scenario S4 --levels b0,b1,b2 --concurrency 1 --warmup 10000 --samples 100000 --repetitions 5` plus explicit host metadata.
- Final package contains 45 JSONL records and passes `audit-rust-s4.py`.

- [x] Run Rust workspace tests, all Python audit suites, S4 protocol tests, and focused burst contracts before the external run.
- [x] Execute the calibrated Rust S4 benchmark against `D:\02-PERSONAL\TOOLS\prism-datasets\p0-100k` using a new immutable output path.
- [x] Parse every JSONL line with `json.loads`, verify 45 unique tuples and phase order, and run `audit-rust-s4.py`.
- [x] Verify correctness, offered/processed rates, lateness metrics, calibration values, per-phase p99 taxes, throughput, dataset digest, B2 evidence, and host metadata.
- [x] Document command, digest, output path, calibration, phase observations, limitations, and explicit non-qualification status.
- [x] Run final Rust/Python suites, `git diff --check`, `git status --short`, and `git ls-files '*100k*'`; confirm no binary fixture is tracked.
- [x] Mark all completed plan steps, append one row per task plus the final plan row to `docs/consumo.md`, and commit with `docs: finalize S4 burst increment`.
- [x] Merge and publish only after final verification is green; report commit and evidence paths without claiming P0 qualification.

## Final Verification

- B0, B1, and B2 each have five valid records in baseline, burst, and recovery order.
- The package contains exactly 45 records and every phase has 100,000 frames.
- Calibration, offered rates, lateness counters, and processed throughput are finite and internally consistent.
- Workflow and observability taxes match phase and repetition.
- The independent audit accepts valid evidence and rejects malformed phase, identity, metric, correctness, calibration, and tax cases.
- No queue, broker, transport, S5, cross-language, or P0 qualification claim is introduced.
