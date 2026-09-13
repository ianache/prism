# Rust S7 Production Vertical Slice — Implementation Plan

> **Execution note:** Implement this plan task-by-task with verification after each task.

**Goal:** Deliver a supported local streaming command, `prism-run`, that processes canonical binary telemetry frames carried in JSONL envelopes from stdin and emits one contractual JSONL result per input line for B0, B1, and observed B2.

**Design:** [S7 design](../specs/2026-09-13-rust-s7-production-vertical-slice-design.md)

## Constraints and invariants

- Preserve the existing binary frame codec, rule precedence, outcome semantics, and B0/B1 tests.
- B2 is B1 plus the existing `Observer` hook; it must not create a second pipeline implementation.
- Keep the live path free of benchmark timers, taxes, queues, brokers, network code, and file output.
- Use only the standard library for the new adapter unless an existing workspace dependency is genuinely required.
- The fixed measurement target remains 100,000 frames for benchmark decisions; S7 is a per-line functional flow and does not claim a 100K performance result.
- Preserve S5/S6 evidence and the existing untracked `graphify-out/` directory.

## Task 1: Add the runner crate and route contract

**Files:** `Cargo.toml`, `crates/prism-runner/Cargo.toml`, `crates/prism-runner/src/lib.rs`, `crates/prism-runner/src/main.rs`, `crates/prism-runner/tests/route_contract.rs`

1. Add `crates/prism-runner` as a workspace member and depend on `prism-runtime` by path.
2. Define a small route enum for `b0`, `b1`, and `b2`, with deterministic parsing and stable startup errors for missing/unknown routes.
3. Define adapter-level request/result types sufficient for the JSONL contract without changing runtime model types.
4. Write failing unit tests first for route parsing, required `--route`, optional request-id prefix, and `--help` behavior.
5. Implement the minimal CLI argument parser and route dispatch shell.
6. Run the focused runner contract tests and `cargo fmt --check`.

## Task 2: Implement the JSONL envelope and hex boundary

**Files:** `crates/prism-runner/src/protocol.rs`, `crates/prism-runner/tests/protocol_contract.rs`

1. Add failing tests for valid envelopes, escaped request IDs, missing fields, duplicate/unknown fields policy, odd/non-hex payloads, blank lines, and deterministic error codes.
2. Implement a bounded, std-only parser for the required JSON string fields `request_id` and `payload_hex`; do not parse arbitrary telemetry JSON or silently accept malformed syntax.
3. Decode hexadecimal payloads into bytes with explicit errors and reject empty payloads before runtime dispatch.
4. Implement JSON string escaping for output fields and ensure every accepted input line can produce exactly one serialized result line.
5. Verify malformed input is represented in-band and does not terminate processing of subsequent lines.

## Task 3: Connect B0/B1/B2 to runtime outcomes

**Files:** `crates/prism-runner/src/dispatch.rs`, `crates/prism-runner/src/output.rs`, `crates/prism-runner/tests/dispatch_contract.rs`

1. Add failing tests using existing binary fixtures for normalized and rejected frames on all routes.
2. Dispatch B0 through `prism_runtime::b0::process`, B1 through `Pipeline::process`, and B2 through `Pipeline::process_observed` with a local observer collecting F1–F6 counts/timings for output diagnostics.
3. Reuse the runtime’s canonical outcome serializer where available; add only the minimal wrapper fields (`request_id`, `ok`, `route`, and `outcome`/`error`).
4. Encode runtime rejections as valid results (`ok: true`) and adapter/parser failures as `ok: false` with stable error codes and messages.
5. Make B2 observation data deterministic in shape and non-authoritative for outcome semantics; it must never alter the returned outcome.
6. Run focused Rust tests and compare representative B0/B1/B2 outcomes with existing fixture expectations.

## Task 4: Implement streaming lifecycle and end-to-end contracts

**Files:** `crates/prism-runner/src/main.rs`, `crates/prism-runner/tests/cli_contract.rs`, `tests/test_s7_vertical_slice.py`, `docs/rust-b0-b1.md`

1. Add failing process-level tests that launch `prism-run`, send multiple lines, close stdin, and assert one output line per input in order.
2. Implement buffered stdin reading, per-line dispatch, immediate stdout flush, clean EOF, and continued processing after in-band malformed lines.
3. Ensure startup/configuration failures exit nonzero; individual malformed frames do not; stdout write/flush failures propagate as process failures.
4. Add a reproducible local smoke command using an existing fixture, without committing generated binary datasets or result files.
5. Document the envelope, route selection, lifecycle, and the explicit S7 non-goals (network/P0/performance qualification).

## Task 5: Full verification and evidence package

**Files:** `docs/evidence/rust-s7-production-vertical-slice-2026-09-13.md`, `docs/consumo.md`

1. Run `cargo test --workspace --release` with the repository’s required Windows release flags if needed, plus all Python tests.
2. Run the S7 process smoke for B0, B1, and B2 and retain only concise, reproducible evidence: command, commit, fixture identity, counts, exit code, and representative result checks.
3. Verify existing S5/S6 evidence and benchmark paths are unchanged; explicitly do not reinterpret their p99 failures as resolved.
4. Review the diff for accidental benchmark, dataset, or generated-file changes.
5. Add one consumption row per S7 task and one plan-completion row to `docs/consumo.md`, using real elapsed time and provider token values when available and `N/D` otherwise.
6. Commit the complete increment only after verification succeeds; do not claim production/P0 readiness.

## Verification commands

From the repository root, use the local Rust toolchain path if `cargo` is not on `PATH`:

```powershell
rtk proxy python -c "& 'C:\Users\ilver\.cargo\bin\cargo.exe' test --workspace --release"
rtk pytest tests/
```

The final smoke command and its exact fixture path will be recorded in the S7 evidence document after implementation.

## Completion criteria

- `prism-run --route b0|b1|b2` is executable locally and documented.
- Valid input produces ordered, one-for-one JSONL output with stable request IDs.
- B0/B1/B2 outcomes preserve runtime semantics; B2 observation is additive only.
- Parser, route, runtime rejection, EOF, flush, and process-level failure contracts pass.
- Full Rust and Python suites pass.
- Evidence and consumption log are updated; no P0 or network claim is made.
