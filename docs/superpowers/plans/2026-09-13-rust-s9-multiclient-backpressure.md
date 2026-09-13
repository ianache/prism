# Rust S9 Multi-Client Concurrency and Backpressure — Implementation Plan

> **Execution note:** Implement task-by-task with verification after each task.

**Goal:** Extend S8 local TCP mode with a bounded worker pool and bounded connection queue, preserving S7/S8 semantics and per-connection ordering.

**Design:** [S9 multi-client and backpressure](../specs/2026-09-13-rust-s9-multiclient-backpressure-design.md)

## Invariants

- Reuse `process_line`; do not duplicate parsing, dispatch, serialization, or B2 observation logic.
- Keep stdin/stdout and serial TCP behavior unchanged when `--workers 1 --connection-queue 0`.
- Use exactly a bounded number of worker threads; never spawn one unbounded thread per accepted connection.
- Preserve request order within each connection; responses from different connections may interleave.
- Admit at most `workers + connection_queue` connections and reject excess capacity explicitly.
- Preserve the 64 KiB line limit and S8 oversized-line behavior.
- Keep loopback/local scope; no broker, durable queue, TLS, authentication, graceful drain, or P0 claim.
- Preserve the fixed 100,000-frame benchmark target and all S5/S6 evidence.

## Task 1: Add bounded-concurrency CLI contract

**Files:** `crates/prism-runner/src/cli.rs`, `crates/prism-runner/tests/route_contract.rs`

1. Add failing tests for `--workers`, `--connection-queue`, defaults, numeric parsing, zero/negative/oversized values, and rejection in stdin/stdout mode.
2. Add typed worker and queue settings with conservative implementation limits.
3. Preserve all S7/S8 argument behavior and startup error codes/messages.
4. Run focused runner tests and package formatting checks.

## Task 2: Build the bounded worker pool

**Files:** `crates/prism-runner/src/transport.rs`, `crates/prism-runner/src/lib.rs`, `crates/prism-runner/tests/transport_contract.rs`

1. Add failing unit tests for capacity math and bounded job admission.
2. Define a connection job channel with fixed capacity and a fixed set of worker threads.
3. Have each worker process a connection serially using the existing bounded line reader and `process_line`.
4. Ensure worker completion frees capacity and thread panics/errors are not silently treated as successful work.
5. Keep the S8 single-worker/zero-queue path behaviorally equivalent.

## Task 3: Implement saturation and connection-level rejection

**Files:** `crates/prism-runner/src/transport.rs`, `crates/prism-runner/src/output.rs`, `crates/prism-runner/tests/transport_contract.rs`

1. Add failing tests that hold active workers, fill the queue, and attempt one additional connection.
2. On full capacity, send one control JSONL error with route, `ok:false`, and `CAPACITY_EXCEEDED`, then close only that socket.
3. Keep the listener alive and able to accept later connections after a worker or queued job becomes available.
4. Ensure capacity rejection is not counted as a runtime frame outcome and uses a deterministic control request ID.
5. Verify malformed and oversized lines remain scoped to their own admitted connection.

## Task 4: Add process-level multi-client and recovery tests

**Files:** `crates/prism-runner/src/main.rs`, `tests/test_s9_multiclient_backpressure.py`, `docs/rust-b0-b1.md`

1. Add a process test launching B0, B1, and B2 with multiple workers and a bounded queue.
2. Connect multiple clients concurrently, send multiple requests per client, and assert per-client ordering, client isolation, and one response per request.
3. Assert B2 valid responses retain six observer events and runtime/parser error semantics remain unchanged.
4. Hold connections open to deterministically reach capacity; assert the excess client receives `CAPACITY_EXCEEDED` and the server remains alive.
5. Close an active client and verify a later client is admitted.
6. Document worker/queue arguments and a reproducible PowerShell/Python local test.

## Task 5: Final audit, evidence, and commit

**Files:** `docs/evidence/rust-s9-multiclient-backpressure-2026-09-13.md`, `docs/consumo.md`

1. Run `cargo test --workspace --release` and all Python tests, including S7/S8 regression tests.
2. Run package formatting validation and `git diff --check`.
3. Review that benchmark code, S5/S6 evidence, generated datasets, and `graphify-out/` are untouched.
4. Record worker count, queue capacity, client/request counts, saturation result, recovery result, test commands, and explicit non-goals in evidence.
5. Add one consumption row per S9 task and one plan-completion row, using real measurements when available and `N/D` otherwise.
6. Commit only after all verification succeeds; do not claim deployment, durability, or P0 readiness.

## Verification commands

```powershell
rtk proxy 'C:\Users\ilver\.cargo\bin\cargo.exe' test --workspace --release
rtk proxy python -m unittest discover -s tests
rtk proxy 'C:\Users\ilver\.cargo\bin\cargo.exe' fmt --package prism-runner -- --check
rtk git diff --check
```

## Completion criteria

- S9 supports bounded multi-client TCP processing with configurable workers and queue.
- Capacity rejection is explicit, recoverable, and does not terminate the listener.
- Ordering is preserved within each connection and S7/S8 behavior remains green.
- B0/B1/B2 outcomes and B2 observations remain unchanged.
- Full suites, evidence, and consumption records are complete.
