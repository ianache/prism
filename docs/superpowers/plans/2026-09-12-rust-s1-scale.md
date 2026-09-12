# Rust S1 Protocol Completion and 100K Validation Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax.

**Goal:** Complete protocol-compliant Rust S1 evidence and validate B0/B1 correctness on an external 100,000-fixture dataset without committing individual binary fixtures.

**Architecture:** Extend `prism-bench` around explicit dataset, metadata, timing, raw-record, and audit boundaries. Keep all fixture I/O, digesting, correctness comparison, serialization, and output writing outside the primary timer; keep B0/B1 runtime behavior unchanged. Use standard-library Rust only and the independent Python generator/oracle for dataset preparation.

**Tech Stack:** Rust stable/Cargo workspace, Rust standard library only, Python 3 dataset generator/verifier, canonical JSONL, Windows MSVC toolchain.

**Spec:** `docs/superpowers/specs/2026-09-12-rust-s1-scale-design.md`, `specification/benchmark-protocol.md`, `specification/acceptance-criteria.md`.

## Global Constraints

- Preserve the frozen seed `0x505249534D5F5631`, payload classes 105/249/501, workload composition 80% valid/5% invalid/15% edge-complex, and Python oracle independence.
- S1 uses 10,000-frame warm-up, 1,000-frame convergence windows, 5% p99 convergence threshold, five measured repetitions, and 10,000 measured frames per repetition.
- The primary timer covers resident frame processing only; exclude dataset I/O, startup, registry/contract loading, expected-result parsing, JSON, correctness comparison, and raw writing.
- Percentiles are nearest-rank over all samples, retaining p50/p95/p99/p99.9/max and negative workflow-tax values.
- No third-party Rust crates, B2/B3, S2–S5, broker, tracing, transport, production workflow, or P0 qualification claim.
- The 100,000 raw binary fixtures remain external; only manifest, digest, command metadata, and selected immutable JSONL evidence may enter Git.
- Every completed plan/task appends one row to `docs/consumo.md` with real timing/tokens when available and `N/D` otherwise.

---

### Task 1: Add protocol dataset model and internal SHA-256 verification

**Files:**
- Create: `crates/prism-bench/src/sha256.rs`
- Modify: `crates/prism-bench/src/dataset.rs`
- Modify: `crates/prism-bench/src/lib.rs`
- Test: `crates/prism-bench/tests/dataset_contract.rs`

**Interfaces:**
- `sha256_hex(bytes: &[u8]) -> String` computes standard SHA-256 using only the Rust standard library.
- `Dataset { fixtures, expected, manifest_digest, dataset_id, fixture_count, payload_counts, validity_counts }` retains resident bytes and protocol metadata.
- `Dataset::load(path: &Path) -> Result<Dataset, DatasetError>` rejects absent manifest/digest, invalid digest format, changed manifest digest, count mismatch, and missing fixture files.

- [ ] **Step 1: Write failing digest and metadata tests.**

```rust
#[test]
fn sha256_matches_nist_ascii_vector() {
    assert_eq!(sha256_hex(b"abc"), "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad");
}

#[test]
fn dataset_rejects_manifest_digest_mismatch() {
    let path = fixture_copy_with_manifest_digest("00");
    assert!(matches!(Dataset::load(&path), Err(DatasetError::DigestMismatch { .. })));
}
```

- [ ] **Step 2: Run the focused tests and confirm they fail because SHA-256 and typed digest errors are absent.**

Run: `cargo test -p prism-bench --test dataset_contract --release`

Expected: compilation failure naming the missing `sha256_hex` or digest error.

- [ ] **Step 3: Implement SHA-256 compression, manifest digest validation, and protocol count extraction.**

Use the standard 64-round SHA-256 constants and big-endian block processing. Read `manifest.sha256`, normalize one trailing LF, compare exactly 64 lowercase hexadecimal characters, and return a typed mismatch error before exposing resident fixtures.

- [ ] **Step 4: Run dataset tests and the existing smoke runner test.**

Run: `cargo test -p prism-bench --test dataset_contract --release` and `cargo test -p prism-bench --test bench_contract --release`.

Expected: all focused tests pass and the existing 300-fixture S1 correctness test remains 300/300.

- [ ] **Step 5: Commit the dataset integrity boundary.**

```text
git add crates/prism-bench/src/sha256.rs crates/prism-bench/src/dataset.rs crates/prism-bench/src/lib.rs crates/prism-bench/tests/dataset_contract.rs
git commit -m "feat: verify benchmark dataset digests in Rust"
```

### Task 2: Implement complete host metadata and protocol CLI configuration

**Files:**
- Modify: `crates/prism-bench/src/metadata.rs`
- Modify: `crates/prism-bench/src/cli.rs`
- Modify: `crates/prism-bench/src/main.rs`
- Test: `crates/prism-bench/tests/cli_contract.rs`

**Interfaces:**
- `Metadata::collect(args: &CliConfig) -> Metadata` returns CPU model, physical/logical cores, RAM, OS/kernel, Rust/compiler version, governor, container limits, affinity, commit, command, run ID, and `N/D` for unavailable values.
- `CliConfig` validates scenario `S1`, levels `b0,b1`, concurrency `1`, positive warmup/samples/repetitions, required metadata flags, and output path.
- CLI rejects unknown levels, invalid scenario, zero sample values, missing metadata, and malformed numeric values before loading or timing.

- [ ] **Step 1: Write failing CLI and metadata tests.**

Test that valid S1 arguments produce `warmup_frames=10000`, `convergence_window=1000`, `convergence_threshold_percent=5`, `repetitions=5`, `measured_frames=10000`, and that missing/invalid fields return typed errors.

- [ ] **Step 2: Run focused tests and confirm failure for missing protocol fields.**

Run: `cargo test -p prism-bench --test cli_contract --release`.

Expected: assertion or compilation failures because protocol fields and `Metadata::collect` are incomplete.

- [ ] **Step 3: Implement config parsing and metadata collection without external crates.**

Use Windows environment/system commands only during setup, never from the timed closure. Preserve explicit user metadata and mark unavailable host metrics `N/D` instead of inventing values.

- [ ] **Step 4: Run CLI tests and verify the documented command parses.**

Run: `cargo test -p prism-bench --test cli_contract --release` and invoke the binary with the documented S1 flags against the 300-fixture corpus using an external output path.

Expected: valid configuration, complete metadata object, and no output generated by tests in `results/raw`.

- [ ] **Step 5: Commit the configuration boundary.**

```text
git add crates/prism-bench/src/metadata.rs crates/prism-bench/src/cli.rs crates/prism-bench/src/main.rs crates/prism-bench/tests/cli_contract.rs
git commit -m "feat: add protocol-complete S1 configuration metadata"
```

### Task 3: Implement warm-up convergence, repetitions, and complete timing metrics

**Files:**
- Modify: `crates/prism-bench/src/percentiles.rs`
- Modify: `crates/prism-bench/src/runner.rs`
- Test: `crates/prism-bench/tests/runner_contract.rs`

**Interfaces:**
- `percentile_nearest_rank(samples: &mut [u128], percentile_thousandths: u32) -> u128` supports 50_000, 95_000, 99_000, and 99_900.
- `RunConfig` contains warm-up target, convergence window, threshold, max warm-up, repetitions, measured frames, and concurrency.
- `run_level(level: Level, dataset: &Dataset, config: &RunConfig) -> Result<RawRun, RunError>` returns per-repetition samples, convergence evidence, p50/p95/p99/p99.9/max, frames/sec, MB/sec, typed rejection count, execution-failure count, and correctness totals.

- [ ] **Step 1: Write failing percentile, convergence, repetition, and throughput tests.**

Use `[1,2,3,4,5]` for p50/p95/p99.9 nearest-rank, a deterministic synthetic warm-up source for the 5% convergence decision, five repetition assertions, and a known byte/frame elapsed pair for throughput.

- [ ] **Step 2: Run runner tests and confirm failure because the current runner has no convergence or repetitions.**

Run: `cargo test -p prism-bench --test runner_contract --release`.

Expected: compilation or assertion failure for the missing fields/behavior.

- [ ] **Step 3: Implement the runner with explicit timer boundaries.**

Warm until both 10,000 frames and two consecutive 1,000-frame p99 windows within 5%, capped by a typed non-convergence error. For each repetition, time only resident frame calls, collect nanoseconds, and perform outcome serialization/correctness comparison after `Instant::elapsed()`.

- [ ] **Step 4: Run runner tests on smoke data and assert no correctness mismatch.**

Run: `cargo test -p prism-bench --test runner_contract --release` and `cargo test -p prism-runtime --test oracle_parity --release`.

Expected: all five repetition records are populated, B0/B1 matches are 300/300, and negative tax values remain representable.

- [ ] **Step 5: Commit timing behavior.**

```text
git add crates/prism-bench/src/percentiles.rs crates/prism-bench/src/runner.rs crates/prism-bench/tests/runner_contract.rs
git commit -m "feat: implement protocol-complete S1 timing"
```

### Task 4: Add immutable raw schema, workflow tax, and atomic write-once output

**Files:**
- Modify: `crates/prism-bench/src/output.rs`
- Modify: `crates/prism-bench/src/main.rs`
- Test: `crates/prism-bench/tests/output_contract.rs`

**Interfaces:**
- `RawRecord` serializes required run, dataset, timing, throughput, correctness, metadata, and tax fields as canonical JSON with LF-separated records.
- `workflow_tax_percent(base_p99: u128, compared_p99: u128) -> f64` returns `(compared-base)/base*100` without clamping negative values.
- `write_once_atomic(path: &Path, records: &[RawRecord]) -> Result<(), OutputError>` refuses existing paths and uses a temporary sibling plus rename.

- [ ] **Step 1: Write failing output tests.**

Assert p99 tax `-10%` is retained, required JSON keys parse, five repetitions per level serialize, existing output remains unchanged, and a failed correctness run leaves no final output.

- [ ] **Step 2: Run output tests and confirm missing schema/tax behavior.**

Run: `cargo test -p prism-bench --test output_contract --release`.

Expected: failure because the current string builder lacks required fields and atomic failure behavior.

- [ ] **Step 3: Implement typed raw records, canonical serialization, tax, and atomic write-once.**

Write only after every level/repetition passes correctness and metadata validation. Include `N/D` for unavailable measurements and preserve the exact command/dataset digest.

- [ ] **Step 4: Validate raw JSONL with a standard-library parser.**

Run: `cargo test -p prism-bench --test output_contract --release` and a Python `json.loads` pass over every generated line.

Expected: valid JSONL, no overwrite, and no partial final file after failure.

- [ ] **Step 5: Commit raw evidence handling.**

```text
git add crates/prism-bench/src/output.rs crates/prism-bench/src/main.rs crates/prism-bench/tests/output_contract.rs
git commit -m "feat: emit immutable protocol-complete S1 records"
```

### Task 5: Add external 100K preparation, parity, and audit commands

**Files:**
- Create: `scripts/prepare-rust-100k.ps1`
- Create: `scripts/audit-rust-s1.py`
- Modify: `crates/prism-bench/tests/bench_contract.rs`
- Modify: `.gitignore`
- Modify: `docs/rust-b0-b1.md`

**Interfaces:**
- PowerShell preparation creates a temporary external artifact directory, runs Python generation with count 100000 and the frozen seed, runs Python verification, and prints the artifact path plus manifest digest without staging fixtures.
- `audit-rust-s1.py` validates JSONL schema, five repetitions per level, required metadata, correctness totals/matches, digest consistency, and workflow-tax arithmetic.
- Integration test accepts `PRISM_100K_DATASET`; when unset it skips with an explicit message, and when set it validates the full 100K corpus without generating files in Git.

- [ ] **Step 1: Write failing audit tests and external-corpus integration contract.**

Test missing dataset, malformed raw JSONL, incorrect repetition count, correctness mismatch, digest mismatch, and a valid externally supplied corpus path.

- [ ] **Step 2: Run tests and confirm the expected missing-external-corpus behavior.**

Run: `cargo test -p prism-bench --test bench_contract --release`.

Expected: the opt-in 100K test reports that `PRISM_100K_DATASET` is not set and remains skipped; malformed audit fixtures fail clearly.

- [ ] **Step 3: Implement preparation and audit scripts.**

Use explicit external paths under `D:\02-PERSONAL\TOOLS\prism-datasets\`, never `results/raw` for binary fixtures. Add generated fixture directories to `.gitignore` and keep only small metadata/evidence files eligible for review.

- [ ] **Step 4: Run the 100K preparation and independent verifier.**

Run: `powershell -File scripts/prepare-rust-100k.ps1` and `python scripts/audit-rust-s1.py --dataset <external-100k> --raw <external-jsonl>`.

Expected: 100,000 fixtures, 80/5/15 category counts, all three payload classes, matching manifest digest, and no files staged under `tests/fixtures`.

- [ ] **Step 5: Run opt-in Rust parity and S1 evidence on 100K.**

Run the documented Rust test/benchmark commands with `PRISM_100K_DATASET=<external-100k>` and an external raw output path. Compare 300-fixture and 100K correctness only; do not claim P0 performance qualification.

- [ ] **Step 6: Commit reproducibility tooling and documentation.**

```text
git add scripts/prepare-rust-100k.ps1 scripts/audit-rust-s1.py crates/prism-bench/tests/bench_contract.rs .gitignore docs/rust-b0-b1.md
git commit -m "test: add external 100K S1 validation workflow"
```

### Task 6: Final verification, consumption record, and review package

**Files:**
- Modify: `docs/rust-b0-b1.md`
- Modify: `results/README.md`
- Modify: `results/raw/README.md`
- Modify: `docs/consumo.md`

**Interfaces:**
- Documentation includes exact commands for smoke and external 100K preparation, Rust tests, S1 execution, JSONL audit, and output immutability.
- Review package states explicit limitations: S1/100K evidence is not P0 qualification and individual 100K binaries remain external.

- [ ] **Step 1: Write documentation acceptance checks.**

Check that every command uses the frozen seed, external dataset location, required metadata, output path policy, and correctness-first ordering.

- [ ] **Step 2: Run the full Rust suite and smoke audit.**

Run: `cargo test --workspace --release` and `python scripts/audit-rust-s1.py --dataset tests/fixtures/p0-smoke --raw <smoke-jsonl>`.

Expected: zero Rust failures, valid JSONL, complete metadata, and 300/300 B0/B1 correctness.

- [ ] **Step 3: Run final 100K audit when the external artifact is available.**

Run the external preparation, parity, benchmark, and audit commands from Task 5; preserve the raw JSONL externally and record its digest/locations in the review notes.

- [ ] **Step 4: Append one measured task row to `docs/consumo.md`.**

Use provider-reported tokens/times when available; otherwise record `N/D` with the measurement-source limitation.

- [ ] **Step 5: Inspect VCS state and verify no 100K binary fixtures are staged.**

Run: `git status --short`, `git diff --cached --name-only`, and `git ls-files tests/fixtures | Measure-Object`.

Expected: only code/docs/small metadata are tracked; external fixture directories are absent.

- [ ] **Step 6: Commit final documentation and review evidence.**

```text
git add docs/rust-b0-b1.md results/README.md results/raw/README.md docs/consumo.md
git commit -m "docs: document protocol-complete S1 and 100K validation"
```

## Final Verification

- `cargo test --workspace --release` passes.
- Smoke corpus remains 300/300 for B0/B1.
- External 100K generation, Python verification, Rust parity, and JSONL audit pass when the artifact is available.
- Five repetitions per level contain all required timing, throughput, correctness, digest, metadata, and workflow-tax fields.
- Existing raw output paths are never overwritten; failed correctness produces no valid final record.
- No 100K binary fixtures are committed, and no P0/B2/B3/S2–S5 claim is made.
