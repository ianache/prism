# Rust S10 Operational Lifecycle — Implementation Plan

> **Execution note:** Implement task-by-task with verification after each task.

**Goal:** Add portable readiness and controlled shutdown to S9 TCP mode, with bounded drain and deterministic exit behavior.

**Design:** [S10 operational lifecycle](../specs/2026-09-13-rust-s10-operational-lifecycle-design.md)

## Invariants

- Reuse S7 `process_line` and S9 worker/backpressure semantics.
- Keep stdin/stdout mode unchanged and preserve S8/S9 behavior when no shutdown file is configured.
- Use a portable file sentinel; the runner never creates, deletes, or modifies it.
- Stop admitting new work after shutdown begins; drain admitted jobs until EOF or deadline.
- Keep the 64 KiB line limit, per-connection ordering, and B0/B1/B2 semantics unchanged.
- Do not add native service registration, broker, persistence, TLS, authentication, or P0 claims.
- Preserve the fixed 100,000-frame benchmark target and all S5/S6 evidence.

## Task 1: Add lifecycle CLI and state contracts

**Files:** `crates/prism-runner/src/cli.rs`, `crates/prism-runner/src/lifecycle.rs`, `crates/prism-runner/tests/route_contract.rs`, `crates/prism-runner/tests/lifecycle_contract.rs`

1. Add failing tests for `--shutdown-file`, `--drain-timeout-ms`, defaults, safe bounds, missing values, and rejection in stdin/stdout mode.
2. Define typed lifecycle states and transitions `Starting → Ready → Draining → Stopped`.
3. Define deterministic deadline calculation and idempotent shutdown requests.
4. Preserve existing route, prefix, listen, worker, and queue validation.
5. Run focused runner tests and package formatting checks.

## Task 2: Add readiness and shutdown signalling

**Files:** `crates/prism-runner/src/lifecycle.rs`, `crates/prism-runner/src/main.rs`, `crates/prism-runner/tests/lifecycle_contract.rs`

1. Add failing tests for sentinel polling, one-time `DRAINING`, and `STOPPED` state reporting.
2. Implement a bounded polling watcher that signals shutdown when the exact sentinel path exists.
3. Emit `STARTING` before setup and `READY <address>` only after listener/workers are initialized.
4. Make repeated sentinel observations harmless and keep sentinel ownership external.
5. Ensure startup/bind/pool failures retain nonzero exits.

## Task 3: Integrate lifecycle with the bounded TCP pool

**Files:** `crates/prism-runner/src/transport.rs`, `crates/prism-runner/src/lifecycle.rs`, `crates/prism-runner/src/main.rs`, `crates/prism-runner/tests/transport_contract.rs`

1. Add failing tests for admission before draining, `SERVER_DRAINING` after transition, and worker/queue completion.
2. Add a shared shutdown signal to the listener and workers without unbounded threads or channels.
3. Stop accepting new jobs when draining begins; reject newly accepted sockets with one control JSONL error and close them.
4. Preserve already admitted active and queued connections for drain processing.
5. Close remaining sockets at the deadline, release permits, join workers, and emit `STOPPED`.
6. Keep S9 capacity rejection distinct from lifecycle rejection (`CAPACITY_EXCEEDED` vs `SERVER_DRAINING`).

## Task 4: Add process-level drain and timeout verification

**Files:** `tests/test_s10_operational_lifecycle.py`, `docs/rust-b0-b1.md`, `docs/evidence/rust-s10-operational-lifecycle-2026-09-13.md`

1. Add a process test that waits for `READY`, holds an active client, creates the sentinel, and verifies `DRAINING`.
2. Verify new connections receive `SERVER_DRAINING` while the admitted client remains usable.
3. Release the active client and assert `STOPPED` with exit code 0.
4. Add a timeout test with an idle admitted client and verify bounded termination plus `STOPPED`.
5. Rerun unchanged S7 stdin/stdout, S8 serial TCP, and S9 multi-client/backpressure tests.
6. Document startup, sentinel ownership, timeout, and local PowerShell usage.

## Task 5: Final audit, consumption record, and commit

**Files:** `docs/consumo.md`, `docs/evidence/rust-s10-operational-lifecycle-2026-09-13.md`

1. Run the complete Cargo release workspace suite and all Python tests.
2. Run runner formatting validation and `git diff --check`.
3. Review that benchmark code, S5/S6 evidence, generated datasets, and `graphify-out/` remain untouched.
4. Record lifecycle states, drain/timeout behavior, request counts, exit code, commands, and explicit non-goals.
5. Append one consumption row per S10 task and one plan-completion row, using measured values when available and `N/D` otherwise.
6. Commit only after verification succeeds; do not claim service deployment or P0 readiness.

## Verification commands

```powershell
rtk proxy 'C:\Users\ilver\.cargo\bin\cargo.exe' test --workspace --release
rtk proxy python -m unittest discover -s tests
rtk proxy 'C:\Users\ilver\.cargo\bin\cargo.exe' fmt --package prism-runner -- --check
rtk git diff --check
```

## Completion criteria

- TCP mode reports readiness and controlled lifecycle states.
- Sentinel shutdown stops new admission and drains admitted work within the configured deadline.
- Timeout closes remaining work deterministically and exits successfully after requested shutdown.
- S7/S8/S9 behavior and B0/B1/B2 outcomes remain green.
- Evidence and consumption records are complete, with no deployment/P0 claim.
