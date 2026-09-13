# Rust S1 Evidence Completeness Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Make Rust S1 raw evidence self-describing and independently auditable while preserving B0/B1 behavior and the external 100K policy.

**Architecture:** Extend the existing dataset, runner, metadata, output, and audit boundaries instead of introducing a new reporting layer. The runner produces typed evidence; the serializer emits deterministic JSONL; the Python audit enforces cross-record invariants.

**Tech Stack:** Rust stable/Cargo standard library, Python 3 standard library, canonical JSONL, Windows MSVC release toolchain.

**Spec:** `docs/superpowers/specs/2026-09-12-rust-s1-evidence-completeness-design.md`, `specification/benchmark-protocol.md`, `specification/acceptance-criteria.md`.

## Global Constraints

- Preserve B0/B1 runtime behavior, frozen seed `0x505249534D5F5631`, payload classes 105/249/501, and 80/5/15 workload composition.
- Preserve the S1 protocol: 10,000 warm-up frames, 1,000-frame windows, 5% p99 convergence threshold, five repetitions, and 10,000 measured frames.
- Keep the primary timer around resident frame processing only; correctness, classification, aggregation, serialization and writing stay outside it.
- Use nearest-rank p50/p95/p99/p99.9/max and retain negative workflow-tax values.
- Use standard-library Rust only; do not add B2/B3, S2–S5, transport, tracing, broker, production workflow, or P0 qualification.
- Keep 100K binary fixtures outside Git and append one consumption row per task to `docs/consumo.md`.

### Task 1: Extend dataset and run evidence models

**Files:**
- Modify: `crates/prism-bench/src/dataset.rs`
- Modify: `crates/prism-bench/src/runner.rs`
- Test: `crates/prism-bench/tests/dataset_contract.rs`
- Test: `crates/prism-bench/tests/runner_contract.rs`

**Interfaces:**
- `Dataset` exposes `dataset_id`, `fixture_count`, `payload_counts`, and `validity_counts`.
- `RawRun` exposes warm-up target/actual frames, convergence window/result, measured frames, typed rejection count, execution-failure count, and correctness totals.

- [ ] Write tests that assert smoke dataset identity/counts and that a run exposes protocol evidence with five repetitions.
- [ ] Run `cargo test -p prism-bench --test dataset_contract --release` and `cargo test -p prism-bench --test runner_contract --release`; confirm the new field assertions fail.
- [ ] Implement the smallest typed fields and propagate manifest counts and runner evidence without changing execution semantics.
- [ ] Re-run both focused suites and assert all new values are internally consistent.
- [ ] Append the task consumption row and commit with `feat: expose complete S1 run evidence`.

### Task 2: Complete raw record and invocation metadata

**Files:**
- Modify: `crates/prism-bench/src/output.rs`
- Modify: `crates/prism-bench/src/metadata.rs`
- Modify: `crates/prism-bench/src/main.rs`
- Test: `crates/prism-bench/tests/output_contract.rs`
- Test: `crates/prism-bench/tests/cli_contract.rs`

**Interfaces:**
- `RawRecord` serializes dataset identity/counts, UTC timestamp, exact command, protocol parameters, convergence evidence, per-repetition samples, correctness/failure counts, and required metadata.
- `Metadata::collect` accepts the parsed invocation context and emits `N/D` for unavailable host values.

- [ ] Add failing JSON assertions for every required field and a CLI test proving the exact command and run identity are captured.
- [ ] Run the focused output and CLI tests to establish the missing-field failures.
- [ ] Implement deterministic serialization using the existing ordered maps and preserve write-once atomic output.
- [ ] Run focused tests plus a 300-fixture CLI smoke command writing to an external temporary JSONL path; parse every line with Python `json.loads`.
- [ ] Append the task consumption row and commit with `feat: emit self-describing S1 raw records`.

### Task 3: Harden the independent S1 audit

**Files:**
- Modify: `scripts/audit-rust-s1.py`
- Test: `scripts/test_audit_rust_s1.py`
- Modify: `docs/rust-b0-b1.md`

**Interfaces:**
- `audit-rust-s1.py` validates required keys, exact B0/B1 repetition cardinality, digest and dataset identity, counts, finite numeric metrics, correctness, convergence, and tax arithmetic.
- The audit exits nonzero with a field-specific error for each invalid invariant.

- [ ] Create compact in-memory/temp-directory audit fixtures for valid data, missing metadata, digest mismatch, wrong repetitions, correctness mismatch, non-finite metric, failed convergence, and incorrect tax.
- [ ] Run `python -m pytest scripts/test_audit_rust_s1.py` and confirm the new invalid cases fail before implementation.
- [ ] Implement explicit validators using only Python standard-library modules and retain the successful 100K audit path.
- [ ] Run the audit test file, the smoke audit, and the external 100K audit; expect valid output only for the complete records.
- [ ] Update the documentation with the exact audit command and append the task consumption row.
- [ ] Commit with `test: enforce complete S1 evidence audit`.

### Task 4: Final verification and review package

**Files:**
- Modify: `results/README.md`
- Modify: `results/raw/README.md`
- Modify: `docs/rust-b0-b1.md`
- Modify: `docs/consumo.md`

- [ ] Add documentation checks for immutable raw input, external 100K binaries, required fields, and limitations.
- [ ] Run `cargo test --workspace --release` with `RUSTFLAGS=-C debuginfo=0`.
- [ ] Run the smoke S1 command and audit its JSONL; confirm B0/B1 correctness and complete metadata.
- [ ] Re-run the existing external 100K verifier, opt-in Rust contract, and audit without regenerating binaries unless the artifact is unavailable.
- [ ] Verify `git status --short`, tracked files, and absence of 100K binaries from Git; preserve unrelated `graphify-out/` state.
- [ ] Append the final task and plan rows to `docs/consumo.md` and commit with `docs: finalize S1 evidence completeness`.

## Final Verification

- `cargo test --workspace --release` passes.
- Smoke and external 100K raw JSONL each contain five B0 and five B1 records with complete fields.
- Python audit accepts valid output and rejects each malformed fixture class.
- Existing output files are never overwritten and no final output is emitted after invalid correctness/evidence.
- No 100K binary fixture is tracked and no P0/B2/B3/S2–S5 claim is made.
