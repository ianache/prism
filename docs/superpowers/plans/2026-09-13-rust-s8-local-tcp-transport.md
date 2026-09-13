# Rust S8 Local TCP Transport — Implementation Plan

> **Execution note:** Implement task-by-task with verification after each task.

**Goal:** Add an optional local TCP mode to `prism-run` while preserving the verified S7 stdin/stdout flow and the existing B0/B1/B2 runtime semantics.

**Design:** [S8 local TCP transport](../specs/2026-09-13-rust-s8-local-tcp-transport-design.md)

## Invariants

- Reuse S7 parser, dispatch, B2 observer, and serializer; do not duplicate protocol semantics.
- Keep the default stdin/stdout mode behavior unchanged.
- Bind loopback by default in documentation; never introduce a broker, queue, TLS, authentication, persistence, or P0 claim.
- Handle one client connection at a time in S8; multi-client concurrency is deferred.
- Enforce a 64 KiB maximum input line and close a connection after an oversized line.
- Preserve the 100,000-frame benchmark decision target, but do not present TCP smoke as performance evidence.
- Preserve S5/S6 evidence and the unrelated `graphify-out/` directory.

## Task 1: Extend CLI and shared line-processing boundary

**Files:** `crates/prism-runner/src/cli.rs`, `crates/prism-runner/src/lib.rs`, `crates/prism-runner/src/main.rs`, `crates/prism-runner/tests/route_contract.rs`

1. Add failing tests for optional `--listen host:port`, mutual exclusion/selection of stdin versus TCP mode, invalid addresses, and port `0`.
2. Add a typed listen configuration and deterministic argument errors without changing existing route/prefix behavior.
3. Extract one shared `process_line(route, prefix, sequence, line)` function that both transports call.
4. Keep startup errors nonzero and preserve request ID fallback/prefix behavior.
5. Run focused runner tests and package formatting checks.

## Task 2: Implement bounded TCP transport

**Files:** `crates/prism-runner/src/transport.rs`, `crates/prism-runner/src/main.rs`, `crates/prism-runner/tests/transport_contract.rs`

1. Add failing unit tests for address binding, 64 KiB line acceptance/rejection, clean EOF, and write/flush error handling.
2. Implement a blocking `TcpListener` transport that binds the configured address and handles accepted connections serially.
3. Read newline-delimited requests with a fixed maximum line size; do not buffer unbounded client input.
4. Emit startup address on stderr only in TCP mode; keep stdout unused by the TCP protocol.
5. Process multiple lines per connection, flush each response, close cleanly on client EOF, and return to accept the next client.
6. Close the connection after `LINE_TOO_LARGE` while leaving the process available for later connections.

## Task 3: Wire TCP mode to S7 processing

**Files:** `crates/prism-runner/src/main.rs`, `crates/prism-runner/src/transport.rs`, `crates/prism-runner/tests/transport_contract.rs`

1. Add failing integration tests that launch `prism-run --route b0|b1|b2 --listen 127.0.0.1:0`, discover the bound port from stderr, and connect via a TCP client.
2. Send two valid fixture envelopes and one malformed envelope over the same connection.
3. Assert exactly one response per request, ordered IDs, `ok`/error fields, and B2 observation events.
4. Assert a malformed request is in-band and the connection remains usable for the next valid request.
5. Verify that runtime rejection remains `ok:true` while parser/hex failures remain `ok:false`.

## Task 4: Verify connection lifecycle and S7 non-regression

**Files:** `tests/test_s8_tcp_transport.py`, `docs/rust-b0-b1.md`, `docs/evidence/rust-s8-local-tcp-transport-2026-09-13.md`

1. Add a Python process-level test for clean client EOF followed by a second client connection.
2. Add coverage for oversized lines and confirm the server closes only that connection.
3. Retain and rerun the existing S7 stdin/stdout E2E test unchanged.
4. Document PowerShell and Python examples for launching the listener and sending JSONL.
5. Document explicit scope limits: single active client, loopback, no TLS/auth/broker/queue/P0/performance claim.

## Task 5: Final audit, consumption record, and commit

**Files:** `docs/consumo.md`, `docs/evidence/rust-s8-local-tcp-transport-2026-09-13.md`

1. Run `cargo test --workspace --release` and all Python tests.
2. Run package formatting validation for `prism-runner` and `git diff --check`.
3. Review the diff to ensure benchmark files, S5/S6 evidence, generated datasets, and `graphify-out/` were not altered.
4. Record command, commit, loopback behavior, request/response counts, exit/lifecycle checks, and test results in the evidence file.
5. Append one consumption row per S8 task and one completion row for the plan, using measured times/tokens when available and `N/D` otherwise.
6. Commit only after verification is green; do not claim production deployment or P0 qualification.

## Verification commands

```powershell
rtk proxy 'C:\Users\ilver\.cargo\bin\cargo.exe' test --workspace --release
rtk proxy python -m unittest discover -s tests
rtk proxy 'C:\Users\ilver\.cargo\bin\cargo.exe' fmt --package prism-runner -- --check
rtk git diff --check
```

## Completion criteria

- `prism-run` supports both S7 stdin/stdout and S8 TCP modes.
- TCP mode accepts multiple JSONL requests per connection and one additional connection after clean EOF.
- Oversized lines are rejected safely and do not terminate the listener process.
- B0/B1/B2 outcomes and B2 observations match S7 behavior.
- Full Rust/Python verification is green and evidence/consumption records are complete.
