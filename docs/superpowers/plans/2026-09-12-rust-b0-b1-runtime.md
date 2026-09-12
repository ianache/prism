# Rust B0/B1 Runtime and S1 Benchmark Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Implement independent Rust B0/B1 telemetry paths and an S1 benchmark harness that match the frozen Python oracle on an immutable smoke corpus.

**Architecture:** Create a standard-library-only Cargo workspace with `prism-runtime` and `prism-bench`. B0 is a direct native path; B1 is a statically registered F1-F6 pipeline. The benchmark loads fixtures before timing and writes immutable raw JSONL outside the measured path.

**Tech Stack:** Rust stable, Cargo workspace, Rust standard library only, Python 3 oracle/generator for fixture preparation, canonical JSONL, no broker or tracing dependency.

**Spec:** `docs/superpowers/specs/2026-09-12-rust-b0-b1-runtime-design.md`; `specification/telemetry-workflow.md`; `specification/benchmark-protocol.md`; `specification/acceptance-criteria.md`.

## Global Constraints

- Preserve protocol v1 little-endian offsets, payload classes 105/249/501, CRC-32C Castagnoli, and F1 order: truncation, magic, version, length, protocol, ranges, checksum.
- Use integer-only arithmetic and fixed-capacity sensor storage for at most 76 records.
- B0 and B1 must return equivalent outcomes but must not share the B1 registry/pipeline implementation.
- The primary timer starts with resident bytes and ends at the Rust outcome; exclude dataset I/O, contract loading, JSON, result writing, and orchestration.
- Use `std::time::Instant` and nearest-rank percentiles; retain negative workflow-tax values.
- Keep Python as an independent oracle; Rust tests may compare against its committed smoke outputs but must not import Rust from Python or call Python during measured runs.
- Do not add third-party Rust crates, B2/B3, transport, tracing, production workflow behavior, or P0 qualification claims.
- Never overwrite an existing raw benchmark output.
- Run commands from the repository root; use `cargo test --workspace` for the full Rust suite.

---

### Task 1: Create the Cargo workspace and bounded domain model

**Files:**
- Create: `Cargo.toml`
- Create: `crates/prism-runtime/Cargo.toml`
- Create: `crates/prism-runtime/src/lib.rs`
- Create: `crates/prism-runtime/src/model.rs`
- Create: `crates/prism-runtime/src/crc32c.rs`
- Create: `crates/prism-runtime/tests/model_contract.rs`

**Interfaces:**
- `SensorRecord { id: u8, kind: u8, value: i32 }`.
- `SensorSet { len: u8, records: [SensorRecord; 76] }` with `empty()`, `push(SensorRecord)`, and `as_slice()`.
- `TelemetryFrame { device_id: u64, timestamp_unix_s: u64, latitude_e7: i32, longitude_e7: i32, speed_cm_per_s: u16, heading_cdeg: u16, ignition: u8, battery_mv: u16, sensors: SensorSet, flags: u8, protocol: u8 }`.
- `Classification`, `Severity`, and `Route` enums with the frozen string values.
- `NormalizedTelemetry`, `RejectionCode`, `RejectionStage`, `RejectionContext`, `Rejection`, `ExecutionFailure`, and `Outcome` enums/structs deriving `Debug`, `Clone`, `PartialEq`, and `Eq`.
- `crc32c(bytes: &[u8]) -> u32`.

- [ ] **Step 1: Write the failing model and CRC contract tests.**

```rust
#[test]
fn sensor_set_rejects_more_than_76_records() {
    let mut sensors = SensorSet::empty();
    for id in 1..=76 { sensors.push(SensorRecord { id, kind: 1, value: 0 }).unwrap(); }
    assert!(sensors.push(SensorRecord { id: 77, kind: 1, value: 0 }).is_err());
}

#[test]
fn crc32c_matches_ascii_vector() {
    assert_eq!(crc32c(b"123456789"), 0xe3069283);
}
```

- [ ] **Step 2: Run the focused tests and confirm they fail because the workspace and types do not exist.**

Run: `cargo test -p prism-runtime --test model_contract -- --nocapture`

Expected: compilation failure naming the missing workspace/types.

- [ ] **Step 3: Add the workspace, fixed-capacity model, enums, outcome types, and reflected CRC-32C implementation.**

Use `SensorRecord::default()` for array initialization. Make `SensorSet::push` return `Result<(), SensorSetError>` and reject a 77th record without allocation.

- [ ] **Step 4: Run the focused tests and the complete runtime unit suite.**

Run: `cargo test -p prism-runtime`

Expected: PASS with zero failures.

- [ ] **Step 5: Commit the bounded domain model.**

```text
git add Cargo.toml crates/prism-runtime
git commit -m "feat: add Rust telemetry domain model"
```

### Task 2: Implement the independent frame decoder and F1 rejection precedence

**Files:**
- Create: `crates/prism-runtime/src/frame.rs`
- Create: `crates/prism-runtime/tests/frame_contract.rs`
- Modify: `crates/prism-runtime/src/lib.rs`

**Interfaces:**
- `decode_frame(payload: &[u8]) -> Result<TelemetryFrame, Rejection>` for the B0 decoder.
- `payload_class(payload_length: usize) -> Option<u16>`.
- `validate_ranges(frame: &TelemetryFrame) -> Result<(), Rejection>`.
- `RejectionContext` must preserve the stable integer/string fields required by `contracts/data/rejection.schema.json`.

- [ ] **Step 1: Write failing tests for exact lengths, valid boundary values, all seven rejection codes, and precedence.**

Tests must include empty/truncated input, bad magic plus bad version, declared length mismatch plus bad protocol, invalid ranges plus bad checksum, sensor ordering, sensor count/area mismatch, and all three payload classes.

- [ ] **Step 2: Run the frame tests and confirm failure.**

Run: `cargo test -p prism-runtime --test frame_contract -- --nocapture`

Expected: compilation or assertion failures because `decode_frame` is absent.

- [ ] **Step 3: Implement little-endian field reads, exact length checks, range validation, strict sensor ordering, and CRC coverage.**

Check the seven conditions in this order: truncation, magic, version, length, protocol, ranges, checksum. Return immediately on the first typed rejection.

- [ ] **Step 4: Run frame tests and compare rejection codes with the Python oracle on hand-built vectors.**

Run: `cargo test -p prism-runtime --test frame_contract -- --nocapture`

Expected: PASS with all precedence assertions green.

- [ ] **Step 5: Commit the decoder.**

```text
git add crates/prism-runtime/src/frame.rs crates/prism-runtime/src/lib.rs crates/prism-runtime/tests/frame_contract.rs
git commit -m "feat: implement Rust telemetry frame decoder"
```

### Task 3: Implement rules and the direct B0 path

**Files:**
- Create: `crates/prism-runtime/src/rules.rs`
- Create: `crates/prism-runtime/src/b0.rs`
- Create: `crates/prism-runtime/tests/b0_contract.rs`
- Modify: `crates/prism-runtime/src/lib.rs`

**Interfaces:**
- `evaluate_rules(frame: &TelemetryFrame) -> RuleEvaluation`.
- `b0::process(payload: &[u8]) -> Outcome`.
- `serialize_outcome_json(outcome: &Outcome) -> String` with sorted canonical keys and LF-free single-record output.

- [ ] **Step 1: Write failing rule and B0 tests.**

Cover R001–R004 equality/threshold behavior, first-match conflicts, missing sensor 1, default classification, valid normalized output, each typed rejection, and unexpected execution failure representation.

- [ ] **Step 2: Run the focused tests and confirm failure.**

Run: `cargo test -p prism-runtime --test b0_contract -- --nocapture`

Expected: failure because rules and `b0::process` are not implemented.

- [ ] **Step 3: Implement the ordered integer rule table as Rust constants and the direct B0 pipeline.**

The B0 path may call the shared decoder and CRC primitive, but it must invoke rules directly and must not construct or traverse a B1 registry.

- [ ] **Step 4: Run B0 tests and the full runtime suite.**

Run: `cargo test -p prism-runtime`

Expected: PASS with zero failures.

- [ ] **Step 5: Commit B0.**

```text
git add crates/prism-runtime/src/rules.rs crates/prism-runtime/src/b0.rs crates/prism-runtime/src/lib.rs crates/prism-runtime/tests/b0_contract.rs
git commit -m "feat: implement Rust B0 telemetry path"
```

### Task 4: Implement the B1 contracts, registry, and sequential F1-F6 pipeline

**Files:**
- Create: `crates/prism-runtime/src/b1.rs`
- Create: `crates/prism-runtime/tests/b1_contract.rs`
- Modify: `crates/prism-runtime/src/lib.rs`

**Interfaces:**
- `FilterId` enum with F1 through F6.
- `FilterDescriptor { id: FilterId, input: &'static str, output: &'static str }`.
- `Pipeline::new() -> Result<Pipeline, RegistryError>` validates the exact ordered F1-F6 descriptors outside the timer.
- `Pipeline::process(&self, payload: &[u8]) -> Outcome`.
- `b1::process(pipeline: &Pipeline, payload: &[u8]) -> Outcome`.

- [ ] **Step 1: Write failing registry and pipeline tests.**

Assert exact filter order, input/output contract names, no side-effect/network capabilities, early F1 rejection, valid F1-F6 output, and B0/B1 equality for representative frames.

- [ ] **Step 2: Run the B1 tests and confirm failure.**

Run: `cargo test -p prism-runtime --test b1_contract -- --nocapture`

Expected: failure because `Pipeline` and B1 filters are absent.

- [ ] **Step 3: Implement static descriptors and typed sequential transitions.**

Validate registry contents in `Pipeline::new`. Keep all registry construction and contract checks outside `process`.

- [ ] **Step 4: Run B1 tests, then the entire runtime suite.**

Run: `cargo test -p prism-runtime`

Expected: PASS with zero failures.

- [ ] **Step 5: Commit B1.**

```text
git add crates/prism-runtime/src/b1.rs crates/prism-runtime/src/lib.rs crates/prism-runtime/tests/b1_contract.rs
git commit -m "feat: implement Rust B1 filter pipeline"
```

### Task 5: Add the immutable 300-fixture smoke corpus and cross-language parity tests

**Files:**
- Create: `tests/fixtures/p0-smoke/fixtures/*.bin`
- Create: `tests/fixtures/p0-smoke/expected-results.jsonl`
- Create: `tests/fixtures/p0-smoke/manifest.json`
- Create: `tests/fixtures/p0-smoke/manifest.sha256`
- Create: `crates/prism-runtime/tests/oracle_parity.rs`
- Create: `scripts/prepare-rust-smoke.ps1`
- Modify: `.gitignore`

**Interfaces:**
- Smoke corpus layout is the same as `tools.dataset.generate --count 300`.
- `oracle_parity` reads fixture files in manifest order, compares canonical Rust outcome JSON with each expected JSONL line, and compares B0 with B1.

- [ ] **Step 1: Write the parity test and a fixture discovery helper before adding fixture bytes.**

The test must fail with a clear missing-corpus error when `tests/fixtures/p0-smoke/manifest.json` is absent.

- [ ] **Step 2: Run the parity test and confirm the expected missing-corpus failure.**

Run: `cargo test -p prism-runtime --test oracle_parity -- --nocapture`

Expected: FAIL with a missing smoke corpus message.

- [ ] **Step 3: Generate the corpus using the repository Python CLI and copy only the 300-fixture artifact into the Rust test location.**

Run from the repository root:

```text
python -m tools.dataset.generate --output <temporary-smoke> --seed 0x505249534D5F5631 --count 300
python -m tools.dataset.verify --dataset <temporary-smoke>
```

Copy `fixtures/`, `expected-results.jsonl`, `manifest.json`, and `manifest.sha256` to `tests/fixtures/p0-smoke/`. Do not copy a generated `__pycache__` or temporary directory.

- [ ] **Step 4: Implement canonical Rust outcome serialization and parity comparison.**

Compare exact canonical JSON lines, not object formatting. Report fixture ID, B0 output, B1 output, and expected line on mismatch.

- [ ] **Step 5: Run parity and all Rust tests.**

Run: `cargo test --workspace`

Expected: every smoke fixture matches the Python oracle and B0 equals B1.

- [ ] **Step 6: Commit the smoke corpus and parity tests.**

```text
git add tests/fixtures/p0-smoke crates/prism-runtime/tests/oracle_parity.rs scripts/prepare-rust-smoke.ps1 .gitignore
git commit -m "test: add Rust oracle parity corpus"
```

### Task 6: Create the S1 benchmark crate and immutable raw JSONL output

**Files:**
- Create: `crates/prism-bench/Cargo.toml`
- Create: `crates/prism-bench/src/main.rs`
- Create: `crates/prism-bench/src/cli.rs`
- Create: `crates/prism-bench/src/dataset.rs`
- Create: `crates/prism-bench/src/percentiles.rs`
- Create: `crates/prism-bench/src/metadata.rs`
- Create: `crates/prism-bench/tests/cli_contract.rs`
- Create: `crates/prism-bench/tests/s1_contract.rs`
- Modify: `Cargo.toml`

**Interfaces:**
- `Dataset::load(path: &Path) -> Result<Dataset, DatasetError>` loads bytes and expected lines before timing.
- `percentile_nearest_rank(samples: &mut [u128], percentile: u32) -> u128`.
- `run_level(level: Level, dataset: &Dataset, config: &RunConfig) -> RawRun`.
- CLI flags: `--dataset`, `--levels b0,b1`, `--scenario S1`, `--warmup`, `--samples`, `--repetitions`, `--output`, and required metadata flags `--cpu-model`, `--cores`, `--ram-bytes`, `--os`, `--governor`, `--affinity`.

- [ ] **Step 1: Write failing tests for nearest-rank percentiles, CLI validation, existing-output rejection, and correctness mismatch.**

Use a five-sample vector to assert p50/p95 nearest-rank behavior and a temporary output path to assert that an existing file is never overwritten.

- [ ] **Step 2: Run benchmark tests and confirm failure.**

Run: `cargo test -p prism-bench`

Expected: compilation failure because the benchmark crate and APIs are absent.

- [ ] **Step 3: Implement dataset loading and digest calculation outside the timed closure.**

Load sorted fixture paths and expected JSONL lines, compute the manifest SHA-256 using an internal standard-library-only implementation, and reject missing metadata or mismatched line counts before warm-up.

- [ ] **Step 4: Implement the timed B0/B1 loop.**

Warm up for `--warmup` iterations. For each measured sample, call the selected level over resident fixtures, record only `Instant` elapsed nanoseconds, and compare outcomes outside the timer. Abort the run on the first correctness mismatch.

- [ ] **Step 5: Implement raw JSONL serialization and write-once output.**

Each record must include run ID, implementation, level, scenario, concurrency, payload class, p50/p95/p99/p99.9/max, frames/sec, MB/sec, correctness totals/matches, metadata, commit, dataset digest, command, and timestamp excluded from the primary timing. Refuse an existing output path.

- [ ] **Step 6: Run benchmark unit, CLI, and integration tests.**

Run: `cargo test --workspace`

Expected: PASS with no correctness mismatches and no output overwrite.

- [ ] **Step 7: Commit the S1 harness.**

```text
git add Cargo.toml crates/prism-bench
git commit -m "feat: add Rust S1 benchmark harness"
```

### Task 7: Document execution and perform final verification

**Files:**
- Create: `docs/rust-b0-b1.md`
- Modify: `results/raw/README.md`
- Modify: `results/README.md`
- Modify: `docs/consumo.md`

**Interfaces:**
- Documentation must reproduce the `cargo test --workspace` and S1 command without network access.
- The document must explicitly state that S1 evidence is not a P0 qualification and that the 1,000,000-file release remains external/pending.

- [ ] **Step 1: Write documentation tests as command snippets and acceptance checklist.**

Include the exact smoke corpus preparation, test command, S1 command with required metadata, raw output location, and failure conditions.

- [ ] **Step 2: Run the clean-machine Rust test command and verify JSONL syntax with a standard-library script.**

Run:

```text
cargo test --workspace
python -c "import json, pathlib; [json.loads(line) for line in pathlib.Path('results/raw').glob('*.jsonl') for line in line.read_text(encoding='utf-8').splitlines()]"
```

Expected: Rust tests pass; every raw JSONL line parses.

- [ ] **Step 3: Run S1 on the smoke corpus with explicit metadata and confirm raw output is created once.**

Run the documented command with `--warmup 10000 --samples 10000 --repetitions 5` and verify correctness totals equal matches for both B0 and B1.

- [ ] **Step 4: Append one measured task row to `docs/consumo.md`.**

Use provider-reported values when available; otherwise use `N/D` and state the measurement source limitation. Do not aggregate independent tasks into one row.

- [ ] **Step 5: Run final verification and inspect the diff.**

Run: `cargo test --workspace`

Expected: PASS, no third-party dependency additions, no B2/B3 files, and no runtime output committed under `results/raw` unless explicitly selected as an immutable evidence artifact.

- [ ] **Step 6: Commit the implementation documentation.**

```text
git add docs/rust-b0-b1.md results/README.md results/raw/README.md docs/consumo.md
git commit -m "docs: document Rust B0 B1 benchmark workflow"
```

## Final Verification

- `cargo test --workspace` passes.
- B0 and B1 match every expected result in the 300-fixture smoke corpus.
- S1 raw JSONL contains required metadata and nearest-rank percentiles.
- Existing raw outputs are never overwritten.
- The primary timer excludes I/O, JSON, contract loading, and result writing.
- No B2/B3, broker, tracing, production runtime, or P0 qualification claim is introduced.
