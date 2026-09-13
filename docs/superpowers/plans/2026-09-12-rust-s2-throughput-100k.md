# Rust S2 Throughput 100K Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add an auditable sequential S2 throughput benchmark for B0, B1, and B2 using 100,000 measured frames per repetition.

**Architecture:** Extend the existing CLI, runner, raw-record serializer, and Python audit rather than creating a new benchmark layer. S2 runs B0/B1/B2 in one matched invocation, computes workflow and observability taxes by repetition, and emits immutable JSONL.

**Tech Stack:** Rust stable/Cargo standard library, Python 3 standard library, canonical JSONL, Windows MSVC release toolchain.

**Spec:** `docs/superpowers/specs/2026-09-12-rust-s2-throughput-100k-design.md`

## Global Constraints

- Use protocol v1.1: 100,000 measured frames, five repetitions, 10,000 warm-up frames, 1,000-frame convergence windows, and 5% p99 convergence threshold.
- Execute B0, B1, and B2 over the identical external 100K corpus and compare matched repetition numbers.
- Preserve B0/B1/B2 logical outputs and correctness requirements.
- Keep dataset loading, oracle comparison, aggregation, serialization, and writing outside the primary timer; B2 observer callbacks remain inside its timer.
- Use nearest-rank p50/p95/p99/p99.9/max and retain negative tax values.
- Keep binary 100K fixtures outside Git and record one consumption row per task plus one final plan row.
- Do not implement S3–S5, B3, concurrency, brokers, Go/Java runtimes, or P0 qualification.

---

### Task 1: Add the S2 protocol and CLI contract

**Files:**
- Modify: `crates/prism-bench/src/cli.rs`
- Modify: `crates/prism-bench/tests/cli_contract.rs`
- Modify: `docs/rust-b0-b1.md`
- Test: `tests/test_s2_protocol.py`

**Interfaces:**
- `parse_args` accepts `--scenario S2`.
- S2 decision mode requires `--samples 100000`, `--repetitions 5`, and levels containing `b0,b1,b2`.
- S2 rejects standalone B2 and any measured-frame target other than 100,000.

- [x] Write failing tests for S2 acceptance, missing B1/B0 rejection, wrong frame target, and documentation of the v1.1 S2 rule.
- [x] Run `python -m unittest tests.test_s2_protocol` and `cargo test -p prism-bench --test cli_contract --release`; confirm expected failures.
- [x] Implement scenario-specific validation while preserving existing S1 compatibility and B2 baseline requirements.
- [x] Run both focused suites and confirm all pass.
- [x] Document the exact S2 command and append the task row to `docs/consumo.md`.
- [x] Commit with `feat: add S2 100K CLI contract`.

### Task 2: Make runner metrics scenario-aware and preserve matched taxes

**Files:**
- Modify: `crates/prism-bench/src/runner.rs`
- Modify: `crates/prism-bench/src/main.rs`
- Modify: `crates/prism-bench/src/output.rs`
- Test: `crates/prism-bench/tests/runner_contract.rs`
- Test: `crates/prism-bench/tests/output_contract.rs`

**Interfaces:**
- `RunConfig` carries `scenario: String` and `measured_frames == 100000` in S2 decision mode.
- `run_level(Level, &Dataset, &RunConfig)` emits five repetition metrics without changing frame processing semantics.
- `RawRecord` contains `scenario: "S2"`, all latency/throughput fields, `workflow_tax_percent`, and `observability_tax_percent`.
- B1 tax uses matching B0 p99; B2 tax uses matching B1 p99.

- [x] Add failing runner/output tests for S2 identity, five repetitions at 100K, B0/B1/B2 tax pairing, finite throughput, and deterministic serialization.
- [x] Run the focused Rust tests and record the expected failures.
- [x] Implement scenario propagation, paired p99 maps, and tax calculation without moving non-processing work into the timer.
- [x] Extend JSON serialization and assert every S2 field through parsed JSON.
- [x] Run focused runner/output suites and the existing B0/B1/B2 contracts.
- [x] Append consumption and commit with `feat: run matched S2 throughput metrics`.

### Task 3: Harden the independent S2 audit

**Files:**
- Create: `scripts/audit-rust-s2.py`
- Create: `scripts/test_audit_rust_s2.py`
- Modify: `docs/rust-b0-b1.md`

**Interfaces:**
- `audit-rust-s2.py --raw <jsonl> --dataset <path>` validates a complete B0/B1/B2 S2 set.
- It requires exactly 15 records: repetitions 1–5 for each level.
- It validates scenario, protocol version, 100K frame count, dataset digest/identity, correctness, finite metrics, B2 F1–F6 evidence, workflow tax, and observability tax.
- It reports the first invalid field/invariant and exits nonzero.

- [x] Create valid in-memory/temp-directory fixtures and invalid cases for missing level, duplicate repetition, wrong frame target, digest mismatch, correctness mismatch, non-finite metric, missing F1–F6 key, wrong workflow tax, and wrong observability tax.
- [x] Run `python -m unittest scripts.test_audit_rust_s2` and confirm failures before implementation.
- [x] Implement standard-library validation and matched p99 arithmetic.
- [x] Run S2 audit tests plus the existing S1 and B2 audit suites.
- [x] Document the audit command, append consumption, and commit with `test: audit S2 throughput evidence`.

### Task 4: Execute the external 100K S2 package

**Files:**
- Modify: `results/reports/README.md`
- Modify: `results/decision-matrix/README.md`
- Modify: `results/raw/README.md`
- Modify: `docs/consumo.md`

**Interfaces:**
- The external package contains one immutable JSONL with 15 records or three separately auditable level files derived from the same invocation.
- Every record reports `scenario: "S2"` and `measured_frames: 100000`.

- [x] Run `cargo test --workspace --release` and all Python audit suites.
- [x] Execute B0/B1/B2 together against `D:\02-PERSONAL\TOOLS\prism-datasets\p0-100k` with 100K samples and five repetitions.
- [x] Parse every JSONL line with `json.loads` and run `audit-rust-s2.py`.
- [x] Verify correctness, p99 taxes, throughput, dataset digest, and five repetitions per level.
- [x] Document the command, digest, output path, limitations, and explicit non-qualification status.
- [x] Append consumption and commit with `docs: publish S2 throughput evidence`.

### Task 5: Final verification and handoff

**Files:**
- Modify: `docs/superpowers/plans/2026-09-12-rust-s2-throughput-100k.md`
- Modify: `docs/consumo.md`

- [x] Run Rust workspace tests, Python audit tests, protocol tests, and the external S2 audit again from the final tree.
- [x] Run `git diff --check`, `git status --short`, and `git ls-files '*100k*'`; confirm no binary fixtures are tracked.
- [x] Mark completed plan steps, append the final plan row to `docs/consumo.md`, and commit with `docs: finalize S2 throughput increment`.
- [x] Report the final commit, evidence paths, test counts, and any limitations without claiming P0 qualification.

## Final Verification

- B0, B1, and B2 each have five valid S2 repetitions.
- Every record contains exactly 100,000 measured frames.
- Correctness is 100% for every measured repetition.
- Workflow and observability taxes are matched by repetition.
- Independent audit accepts valid evidence and rejects all malformed fixture classes.
- No concurrency, transport, broker, S3–S5, or P0 qualification claim is introduced.
