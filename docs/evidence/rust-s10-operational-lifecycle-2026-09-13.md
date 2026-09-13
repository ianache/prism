# S10 Operational Lifecycle — Evidence

**Date:** 2026-09-13  
**Status:** controlled local lifecycle verified; not a deployment or P0 qualification

## Implemented behavior

TCP mode optionally accepts `--shutdown-file <path>` and
`--drain-timeout-ms <ms>`. With the sentinel configured, the process reports
`STARTING`, `READY <address>`, `DRAINING`, and `STOPPED` on stderr. The sentinel
is owned by the supervisor and is never modified by the runner.

On shutdown, new connections receive `SERVER_DRAINING`; admitted connections
remain in the bounded S9 pool until EOF or the drain deadline. Idle connections
are closed after the configured deadline and the process exits successfully.

## Verification

- Rust workspace release suite: 59 tests passed, 0 failed.
- Python suite: 43 tests passed, 0 failed.
- S10 lifecycle tests: 2 passed (drain after EOF and idle timeout).
- S7 stdin/stdout, S8 serial TCP, and S9 multi-client/backpressure tests remained green.
- Runner formatting check and `git diff --check`: passed.
- Drain test observed `SERVER_DRAINING`, then `STOPPED` with exit code 0.
- Timeout test observed bounded termination and `STOPPED` with exit code 0.

No generated dataset or binary fixture was added to Git. S5/S6 evidence and
their unresolved p99 findings are unchanged.

## Boundaries

The control mechanism is a portable local file sentinel. Native service
registration, OS signal adapters, TLS, authentication, persistence, broker,
remote deployment, graceful production supervision, and P0/performance
qualification remain out of scope. The 100,000-frame target remains reserved
for benchmark decisions.
