# Rust B2 Observability and 100K Baseline Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add locally collected B2 filter observability and make 100,000 measured frames per repetition the normative baseline for all later benchmark decisions.

**Architecture:** Extend the existing Rust runtime with a no-op-compatible `Observer` boundary and an observed pipeline entry point. Keep B0/B1 behavior unchanged, collect B2 evidence in the benchmark runner, serialize deterministic raw JSONL, and validate the evidence independently with Python.

**Tech Stack:** Rust stable/Cargo standard library, Python 3 standard library, canonical JSONL, Windows MSVC release toolchain.

**Spec:** `docs/superpowers/specs/2026-09-12-rust-b2-observability-design.md`

## Global Constraints

- Use `local_metrics` as the only B2 variant; do not add OpenTelemetry, brokers, network, persistence, or exporters.
- Preserve B0/B1 logical outcomes, frozen seed `0x505249534D5F5631`, payload classes 105/249/501, and workload composition 80/5/15.
- Use exactly 100,000 measured frames per repetition for B2 and all later comparable decisions, five repetitions, 10,000 warm-up frames, 1,000-frame convergence windows, and a 5% p99 threshold.
- Keep correctness comparison, aggregation, serialization, and writing outside the primary timer; instrumentation callbacks are inside the B2 timer because their cost is being measured.
- Use nearest-rank p50/p95/p99/p99.9/max and retain negative observability-tax values.
- Keep 100K binary fixtures outside Git; update `docs/consumo.md` with one row per completed task and one final plan row.
- Do not implement B3 transport, S3/S4/S5, durable execution, P0 qualification, or the 1,000,000-frame corpus.

---

### Task 1: Version the protocol and establish the 100K decision baseline

**Files:**
- Modify: `specification/benchmark-protocol.md`
- Modify: `specification/acceptance-criteria.md`
- Modify: `docs/rust-b0-b1.md`
- Test: `tests/test_b2_protocol.py`

**Interfaces:**
- The protocol declares `protocol_version: 1.1` and `measured_frames: 100000` for every future comparable decision.
- The acceptance criteria distinguish historical 10K S1 evidence from the new 100K decision baseline.

- [ ] **Step 1: Write the failing protocol contract test**

  Create a standard-library `unittest` that reads the three documents and asserts they contain the versioned 100K rule, the replacement of the 1,000,000-frame S2 target, and the historical-status statement for 10K S1 evidence.

- [ ] **Step 2: Run the test to verify it fails**

  Run:

  ```text
  python -m unittest tests.test_b2_protocol
  ```

  Expected: FAIL because the current protocol still declares the S2 target as 1,000,000 frames and has no versioned 100K rule.

- [ ] **Step 3: Update the protocol documents minimally**

  Change the benchmark protocol to version 1.1, replace S2's measured-frame target with 100,000, and state that 100,000 measured frames per repetition applies to B2 and every later comparable decision. Update acceptance criteria and Rust benchmark documentation without rewriting historical results.

- [ ] **Step 4: Run the contract test to verify it passes**

  Run:

  ```text
  python -m unittest tests.test_b2_protocol
  ```

  Expected: PASS.

- [ ] **Step 5: Record and commit**

  Append the task row to `docs/consumo.md`, then run `git diff --check` and commit:

  ```text
  git add specification/benchmark-protocol.md specification/acceptance-criteria.md docs/rust-b0-b1.md tests/test_b2_protocol.py docs/consumo.md
  git commit -m "docs: set 100K benchmark decision baseline"
  ```

### Task 2: Add the observer boundary without changing B1

**Files:**
- Modify: `crates/prism-runtime/src/b1.rs`
- Modify: `crates/prism-runtime/src/lib.rs`
- Test: `crates/prism-runtime/tests/b1_contract.rs`

**Interfaces:**
- `pub trait Observer { fn on_filter(&mut self, filter: FilterId, elapsed_ns: u128, outcome: ObservationOutcome); }`
- `pub enum ObservationOutcome { Completed, Rejected, ExecutionFailure }`.
- `Pipeline::process(&self, payload: &[u8]) -> Outcome` remains unchanged for B1.
- `Pipeline::process_observed<O: Observer>(&self, payload: &[u8], observer: &mut O) -> Outcome` returns the same outcome while emitting fixed F1–F6 callbacks.

- [ ] **Step 1: Write the failing observer contract tests**

  Add tests that use a recording observer to assert that observed processing returns the same serialized outcome as `process`, records only nonzero invocations for reached filters, emits nonnegative timing values, and preserves the fixed F1–F6 filter identity order.

- [ ] **Step 2: Run the focused tests to verify they fail**

  Run:

  ```text
  cargo test -p prism-runtime --test b1_contract --release
  ```

  Expected: FAIL because `Observer`, `ObservationOutcome`, and `process_observed` do not exist.

- [ ] **Step 3: Implement the smallest compatible observer boundary**

  Add the public observer types, make `process` delegate through a no-op observer, and implement `process_observed`. Factor the current pipeline into six private stage functions matching F1 validate, F2 decode, F3 normalize, F4 evaluate rules, F5 classify, and F6 route; each stage must preserve the current types and final `Outcome`. Use `Instant` around each stage and emit zero callbacks for filters not reached after a typed rejection. Do not change the non-observed output path.

- [ ] **Step 4: Run focused and regression tests**

  Run:

  ```text
  cargo test -p prism-runtime --test b1_contract --release
  cargo test -p prism-runtime --test frame_contract --release
  cargo test -p prism-runtime --test oracle_parity --release
  ```

  Expected: all tests PASS with unchanged B1 oracle output.

- [ ] **Step 5: Record and commit**

  Append the task row to `docs/consumo.md` and commit:

  ```text
  git add crates/prism-runtime/src/lib.rs crates/prism-runtime/src/b1.rs crates/prism-runtime/tests/b1_contract.rs docs/consumo.md
  git commit -m "feat: add local B2 observer boundary"
  ```

### Task 3: Collect B2 metrics at the 100K runner boundary

**Files:**
- Modify: `crates/prism-bench/src/runner.rs`
- Modify: `crates/prism-bench/src/main.rs`
- Modify: `crates/prism-bench/src/output.rs`
- Modify: `crates/prism-bench/src/cli.rs`
- Test: `crates/prism-bench/tests/runner_contract.rs`
- Test: `crates/prism-bench/tests/output_contract.rs`
- Test: `crates/prism-bench/tests/cli_contract.rs`

**Interfaces:**
- `Level` gains `B2` and accepts `b2` in the CLI.
- Decision-mode CLI runs validate `measured_frames == 100_000` for B2 and all future comparable runs; small in-memory unit-test configurations remain available for fast contract tests.
- `RawRun` exposes `observability_variant`, `execution_id`, six-key filter timing/invocation/rejection/failure maps, and `observability_tax_percent` at repetition scope.
- `RawRecord` serializes those fields plus `protocol_version: "1.1"` and `measured_frames: 100000`.
- `workflow_observability_tax_percent(base_p99: u128, compared_p99: u128) -> f64` preserves negative values and returns the matched B2/B1 p99 tax.

- [ ] **Step 1: Write failing runner, output, and CLI tests**

  Add tests for accepted `b2`, rejection of B2 configurations whose measured frame count is not 100,000, five B2 repetitions, exactly six filter keys, per-filter invocation totals, JSON serialization of the new fields, and matched-repetition tax arithmetic.

- [ ] **Step 2: Run focused tests to verify they fail**

  Run:

  ```text
  cargo test -p prism-bench --test runner_contract --release
  cargo test -p prism-bench --test output_contract --release
  cargo test -p prism-bench --test cli_contract --release
  ```

  Expected: FAIL because B2 and its evidence fields are not implemented.

- [ ] **Step 3: Implement B2 runner collection**

  Route B2 through `Pipeline::process_observed`, collect per-filter metrics in a fixed `BTreeMap`, assign a record-scoped execution ID, keep correctness checks after the timed loop, and compute B2 tax against the B1 p99 with the same repetition number.

- [ ] **Step 4: Implement deterministic raw serialization**

  Extend `RawRecord::to_json` with the protocol version, variant, execution ID, filter maps, and observability tax. Preserve ordered keys, atomic write-once behavior, and `N/D` host metadata.

- [ ] **Step 5: Run focused tests to verify they pass**

  Run the three focused suites from Step 2 and expect PASS with no warnings.

- [ ] **Step 6: Record and commit**

  Append the task row to `docs/consumo.md` and commit:

  ```text
  git add crates/prism-bench/src crates/prism-bench/tests docs/consumo.md
  git commit -m "feat: collect B2 observability evidence"
  ```

### Task 4: Extend the independent B2 audit

**Files:**
- Modify: `scripts/audit-rust-s1.py`
- Create: `scripts/audit-rust-b2.py`
- Create: `scripts/test_audit_rust_b2.py`
- Modify: `docs/rust-b0-b1.md`

**Interfaces:**
- `audit-rust-b2.py --dataset <path> --baseline <b1.jsonl> --raw <b2.jsonl>` exits 0 only for five matched B2/B1 repetitions with complete evidence.
- The validator rejects missing fields, wrong protocol version, non-100K frame counts, missing F1–F6 keys, inconsistent invocation totals, non-finite/negative values, correctness mismatches, and incorrect per-repetition tax.
- Existing S1 audit behavior remains unchanged for historical S1 raw records.

- [ ] **Step 1: Write invalid and valid audit fixtures**

  Use `unittest`, temporary directories, and in-memory JSON objects to cover valid B1/B2 records, missing observer metadata, wrong frame target, missing filter key, invocation mismatch, non-finite timing, correctness mismatch, wrong repetition pairing, and incorrect observability tax.

- [ ] **Step 2: Run audit tests to verify the new cases fail**

  Run:

  ```text
  python -m unittest scripts.test_audit_rust_b2
  ```

  Expected: FAIL because the new validator does not exist.

- [ ] **Step 3: Implement explicit standard-library validators**

  Validate required keys and types first, then identity/protocol/frame counts, repetition cardinality, per-filter invariants, correctness, finite numeric metrics, and matched B2/B1 tax. Report the first failing field or invariant by name.

- [ ] **Step 4: Run audit tests and historical audit tests**

  Run:

  ```text
  python -m unittest scripts.test_audit_rust_b2
  python -m unittest discover -s scripts -p test_audit_rust_s1.py
  ```

  Expected: all tests PASS.

- [ ] **Step 5: Update documentation and commit**

  Document the B2 audit command, the 100K rule, and the distinction between historical S1 and current decision evidence. Append the task row and commit:

  ```text
  git add scripts/audit-rust-b2.py scripts/test_audit_rust_b2.py scripts/audit-rust-s1.py docs/rust-b0-b1.md docs/consumo.md
  git commit -m "test: audit complete B2 observability evidence"
  ```

### Task 5: Run the 100K B1/B2 decision package

**Files:**
- Modify: `results/README.md`
- Modify: `results/raw/README.md`
- Modify: `results/reports/README.md`
- Modify: `results/decision-matrix/README.md`
- Modify: `docs/consumo.md`

**Interfaces:**
- The decision package contains separate immutable B1 and B2 raw JSONL files, each with five repetitions over 100,000 measured frames.
- The report records correctness, p50/p95/p99/p99.9/max, throughput, per-filter evidence, and observability tax without claiming P0 qualification.

- [ ] **Step 1: Run the complete Rust suite**

  Run:

  ```text
  cargo test --workspace --release
  ```

  Expected: PASS.

- [ ] **Step 2: Generate a 100K B1 baseline**

  Run the existing CLI against the external 100K corpus with `--levels b1`, `--warmup 10000`, `--samples 100000`, `--repetitions 5`, and a new write-once raw output path. Supply explicit host metadata or `N/D` values.

- [ ] **Step 3: Generate the 100K B2 evidence**

  Run the CLI with the same corpus, warm-up, sample count, repetitions, metadata, and a separate immutable output path using `--levels b2`.

- [ ] **Step 4: Audit both records and verify JSONL integrity**

  Parse every line with `json.loads`, run the B2 audit against the B1 baseline, and verify five repetitions, 100,000 measured frames, correctness parity, complete F1–F6 fields, and matched tax arithmetic.

- [ ] **Step 5: Record the decision package**

  Document paths, dataset digest, commands, limitations, and whether the provisional observability target is met. Do not convert the result into a P0 qualification.

- [ ] **Step 6: Verify repository boundaries**

  Run:

  ```text
  git status --short
  git ls-files '*100k*'
  git diff --check
  ```

  Expected: no binary 100K fixtures are tracked, no unrelated `graphify-out/` state is modified, and the worktree contains only intended source, test, documentation, and consumption-log changes.

- [ ] **Step 7: Append final task and plan rows and commit**

  Append one final task row and one `plan completo` row to `docs/consumo.md`, then commit:

  ```text
  git add results docs/consumo.md
  git commit -m "docs: finalize B2 observability decision package"
  ```

## Final Verification

- `cargo test --workspace --release` passes.
- B1 and B2 each have five valid 100K repetitions over the identical external corpus.
- The independent B2 audit accepts valid evidence and rejects every malformed fixture class.
- B2 correctness matches B1 for all measured fixtures.
- Observability tax is calculated per matching repetition and negative values remain visible.
- No 1,000,000-frame target or P0 qualification is claimed.
- No external observability service, broker, transport, or 100K binary fixture is added to Git.
