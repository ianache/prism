# Rust S3 Concurrency Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add an auditable Rust S3 concurrency benchmark for B0, B1, and B2 over 100,000 measured frames at seven worker counts.

**Architecture:** Extend the existing standard-library runner and JSONL output. The CLI accepts a deterministic concurrency list, the runner executes one scoped worker per partition for each level/concurrency/repetition, and `main.rs` pairs B0/B1/B2 p99 values by both concurrency and repetition before serialization. A Python audit validates the complete 105-record package independently.

**Tech Stack:** Rust stable/Cargo standard library, Python 3 standard library, canonical JSONL, Windows MSVC release toolchain.

**Spec:** `docs/superpowers/specs/2026-09-12-rust-s3-concurrency-design.md`

## Global Constraints

- Use protocol v1.1 and exactly 100,000 measured frames per repetition.
- Use exactly five measured repetitions for each level and concurrency.
- Use concurrency values `1,2,4,8,16,32,64` workers.
- Produce exactly 105 records: 3 levels × 7 concurrency values × 5 repetitions.
- Partition frame indexes deterministically with `index % concurrency == worker_id`.
- Use one owned B1 pipeline and one owned B2 collector per worker.
- Keep loading, correctness comparison, aggregation, serialization, and writing outside the primary measured interval.
- Use synchronized wall-clock timing for throughput and per-frame samples for latency.
- Pair workflow and observability taxes by identical concurrency and repetition; preserve negative values.
- Keep binary fixtures outside Git and make raw output write-once.
- Do not implement S4/S5, brokers, transports, persistent pools, cross-language runs, or P0 qualification.

---

### Task 1: Define the S3 CLI and output cardinality contract

**Files:**
- Modify: `crates/prism-bench/src/cli.rs`
- Modify: `crates/prism-bench/src/main.rs`
- Modify: `crates/prism-bench/tests/cli_contract.rs`
- Create: `tests/test_s3_protocol.py`
- Modify: `docs/rust-b0-b1.md`

**Interfaces:**
- `Config` exposes `concurrencies: Vec<usize>` parsed from `--concurrency 1,2,4,8,16,32,64`.
- S1/S2 scalar invocations remain accepted and map to a one-element vector.
- S3 requires scenario `S3`, levels `b0,b1,b2`, samples `100000`, repetitions `5`, and only the seven allowed worker counts.
- `main.rs` iterates the concurrency vector and appends all level records into one output package.

- [x] Add failing Rust/Python tests for S3 acceptance, missing level rejection, invalid worker count rejection, duplicate worker rejection, wrong frame target, and expected 105-record cardinality.
- [x] Run `python -m unittest tests.test_s2_protocol` and `cargo test -p prism-bench --test cli_contract --release`; confirm the new tests fail for the missing S3 behavior.
- [x] Implement list parsing, S3-specific validation, and S1/S2 compatibility without changing existing B2 baseline validation.
- [ ] Add deterministic run IDs containing scenario, level, concurrency, and repetition, for example `S3-b1-c8-r3`.
- [x] Run the focused protocol and CLI suites and confirm all pass.
- [x] Document the exact S3 command and append the task consumption row.
- [x] Commit with `feat: add S3 concurrency CLI contract`.

### Task 2: Implement deterministic scoped-worker execution

**Files:**
- Modify: `crates/prism-bench/src/runner.rs`
- Modify: `crates/prism-bench/tests/runner_contract.rs`
- Modify: `crates/prism-runtime/src/b1.rs` only if the owned pipeline API requires a narrowly scoped `Send`/ownership adjustment

**Interfaces:**
- `RunConfig` retains one `concurrency: usize` for a single runner invocation and accepts S3 values.
- `run_level(level: Level, dataset: &Dataset, config: &RunConfig) -> Result<RawRun, RunError>` remains the public entry point.
- Add a private worker result containing worker ID, ordered frame samples, correctness counters, and B2 collector maps.
- Reject `concurrency == 0`; reject values other than `1` for S1/S2 and the seven allowed values for S3.

- [x] Add failing tests for deterministic modulo partitioning, exactly one worker per partition, complete 100K frame coverage, worker-owned B1/B2 state, join failure propagation, and stable merged sample order.
- [x] Run `cargo test -p prism-bench --test runner_contract --release`; confirm the new concurrency tests fail before implementation.
- [x] Implement the worker function using `std::thread::scope`, `std::sync::Barrier`, and `std::thread::ScopedJoinHandle`; worker `w` processes only indexes where `index % concurrency == w`.
- [x] Start the synchronized wall-clock timer immediately around the barrier-coordinated processing phase and stop it after all joins; do not include dataset load, expected-output comparison, percentile calculation, or JSON writing.
- [x] Give each non-B0 worker its own `Pipeline`; give each B2 worker its own `Collector`; merge returned counters and evidence in worker ID order.
- [x] Preserve warm-up convergence, five repetitions, typed rejection counts, execution failures, and correctness mismatch abort behavior.
- [x] Run runner contracts plus the existing B0/B1/B2 runtime contracts and confirm all pass.
- [x] Append the task consumption row and commit with `feat: run deterministic S3 workers`.

### Task 3: Serialize S3 metrics and pair taxes by concurrency

**Files:**
- Modify: `crates/prism-bench/src/main.rs`
- Modify: `crates/prism-bench/src/output.rs`
- Modify: `crates/prism-bench/tests/output_contract.rs`
- Modify: `crates/prism-bench/tests/bench_contract.rs`

**Interfaces:**
- `RawRecord` continues to serialize `scenario`, `concurrency`, all latency/throughput metrics, correctness counters, and B2 evidence.
- Baseline maps use `BTreeMap<(usize, usize), u128>` keyed by `(concurrency, repetition)`.
- `workflow_tax_for_repetition` gains a concurrency-aware variant that returns the tax for the exact `(concurrency, repetition)` key and returns `0.0` only for B0.
- S3 throughput uses the worker-group elapsed interval; latency percentiles use the merged per-frame samples.

- [ ] Add failing output tests that parse JSON and assert scenario S3, concurrency, run ID, all five repetitions, finite throughput, and every required B2 map key F1–F6.
- [ ] Add failing tax tests proving B1 uses the matching B0 p99 at the same concurrency and repetition, while B2 uses matching B1 values.
- [ ] Run the focused output and benchmark contracts and record expected failures.
- [ ] Implement the keyed baseline maps and use them when building 105 `RawRecord` values.
- [ ] Preserve negative workflow and observability tax values and deterministic BTreeMap JSON key ordering.
- [ ] Run focused output/benchmark suites and all existing contracts.
- [ ] Append the task consumption row and commit with `feat: serialize matched S3 metrics`.

### Task 4: Build the independent S3 audit

**Files:**
- Create: `scripts/audit-rust-s3.py`
- Create: `scripts/test_audit_rust_s3.py`
- Modify: `docs/rust-b0-b1.md`

**Interfaces:**
- `audit-rust-s3.py --dataset <path> --raw <jsonl>` exits nonzero on the first invalid invariant and prints `ok rows=105 levels=b0,b1,b2 concurrencies=1,2,4,8,16,32,64 frames=100000` for valid evidence.
- The audit requires exactly one `(level, concurrency, repetition)` tuple for all 105 combinations.
- It validates dataset ID/digest, protocol, frame target, correctness, finite nonnegative throughput/latency metrics, F1–F6 evidence, and keyed tax arithmetic.

- [ ] Create valid temporary fixtures and invalid cases for missing tuple, duplicate tuple, invalid concurrency, wrong scenario, wrong frame count, digest mismatch, correctness mismatch, non-finite metric, incomplete F1–F6 evidence, wrong workflow tax, and wrong observability tax.
- [ ] Run `python -m unittest scripts.test_audit_rust_s3` and confirm the new tests fail before implementation.
- [ ] Implement standard-library JSONL parsing, manifest identity checks, tuple cardinality checks, and `(concurrency, repetition)` p99 tax calculations.
- [ ] Run S3 audit tests together with S1 and S2 audit suites.
- [ ] Document the audit command, append the task consumption row, and commit with `test: audit S3 concurrency evidence`.

### Task 5: Execute, verify, and publish the S3 evidence package

**Files:**
- Modify: `results/reports/README.md`
- Modify: `results/decision-matrix/README.md`
- Modify: `results/raw/README.md`
- Modify: `docs/consumo.md`
- Modify: `docs/superpowers/plans/2026-09-12-rust-s3-concurrency.md`

**Interfaces:**
- External output: `D:\02-PERSONAL\TOOLS\prism-datasets\p0-100k-s3.jsonl`; if that path exists, stop and choose a new explicitly named immutable path before running.
- Exact command uses `--scenario S3 --levels b0,b1,b2 --concurrency 1,2,4,8,16,32,64 --warmup 10000 --samples 100000 --repetitions 5` plus explicit host metadata.
- Final package passes `audit-rust-s3.py` and contains 105 records.

- [ ] Run Rust workspace tests, all Python audit suites, S3 protocol tests, and focused concurrency contracts before the external run.
- [ ] Execute the combined Rust S3 benchmark against `D:\02-PERSONAL\TOOLS\prism-datasets\p0-100k` using all seven concurrency values and a new immutable output path.
- [ ] Parse every JSONL line with `json.loads`, verify the 105 unique tuples, and run `audit-rust-s3.py`.
- [ ] Verify correctness, per-concurrency p99 taxes, throughput, latency percentiles, dataset digest, B2 F1–F6 counts, and host metadata.
- [ ] Document command, digest, output path, scaling observations, limitations, and explicit non-qualification status.
- [ ] Run the final Rust/Python suites, `git diff --check`, `git status --short`, and `git ls-files '*100k*'`; confirm no binary fixture is tracked.
- [ ] Mark all completed plan steps, append one row per task plus the final plan row to `docs/consumo.md`, and commit with `docs: finalize S3 concurrency increment`.
- [ ] Merge and publish only after the final verification is green; report commit and evidence paths without claiming P0 qualification.

## Final Verification

- B0, B1, and B2 each have five valid records at every allowed concurrency.
- The package contains exactly 105 records and every record has 100,000 measured frames.
- Correctness is complete for every tuple.
- Workflow and observability taxes match both concurrency and repetition.
- The independent audit accepts valid evidence and rejects every malformed fixture class.
- No S4/S5, broker, transport, cross-language, persistent-pool, or P0 qualification claim is introduced.
