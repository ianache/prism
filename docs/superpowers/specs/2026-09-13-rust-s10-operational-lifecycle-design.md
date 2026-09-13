# Rust S10 Operational Lifecycle Design

**Status:** Proposed for review  
**Date:** 2026-09-13  
**Scope:** Add readiness, controlled shutdown, and bounded drain to the local TCP runner.

## Goal

Make the S9 local TCP process safely manageable: expose explicit lifecycle
states, stop admitting new connections during shutdown, drain admitted work,
and terminate within a configured deadline without silently dropping the
process state.

## Relationship to S9

S9 remains the concurrency and backpressure implementation. S10 wraps its
listener and worker pool with lifecycle control without changing S7 request
semantics, S8 framing, S9 capacity rules, B0/B1/B2 outcomes, or benchmark
paths. Stdin/stdout mode remains unchanged.

## Chosen control interface

TCP mode accepts two optional arguments:

```text
--shutdown-file <path>
--drain-timeout-ms <milliseconds>
```

When `--shutdown-file` is configured, a background watcher requests shutdown
when the exact file exists. The runner never deletes or modifies the sentinel;
the supervisor owns its lifecycle. `--drain-timeout-ms` defaults to 5,000 ms
and has a safe upper bound. These options are invalid in stdin/stdout mode.

This file-sentinel control is intentionally portable and deterministic for the
current local slice. OS service integration and native signal handling remain
future adapters.

## Lifecycle states

The process reports the following states to stderr, one line each:

```text
STARTING
READY <bound-address>
DRAINING
STOPPED
```

`READY` is emitted only after the listener and worker pool are initialized.
While `DRAINING`, the listener stops accepting new work. Existing admitted
connections continue until clean EOF or the drain deadline. After the deadline
the process closes remaining client sockets, joins workers, emits `STOPPED`,
and exits successfully. Startup/bind/pool failures remain nonzero exits.

## Shutdown and admission semantics

- The watcher polls the configured sentinel at a bounded interval.
- Shutdown is idempotent; repeated observations do not restart draining.
- New connections after `DRAINING` are closed with a control JSONL error using
  code `SERVER_DRAINING`, then the listener stops accepting.
- Connections already admitted before `DRAINING` retain S9 ordering and
  backpressure behavior.
- Queued but not yet processed connections are considered admitted and are
  drained before the deadline; they are not silently discarded.
- At deadline, remaining sockets are force-closed and the exit remains
  successful because shutdown was requested and bounded.
- Without `--shutdown-file`, S9 behavior remains unchanged until the host
  terminates the process.

## Architecture

```text
startup → READY
             ↓ sentinel
          DRAINING → stop accept → drain admitted jobs → STOPPED
                                      ↓ deadline
                                  force-close → STOPPED
```

The lifecycle controller owns state, sentinel watching, deadline, and worker
shutdown. The transport owns accept gating and connection close behavior. S7
continues to own line parsing, dispatch, B2 observation, and serialization.

## Verification

- Unit contracts cover lifecycle transitions, option validation, idempotence,
  and deadline calculation.
- Process tests launch S10, wait for `READY`, hold an active client, create the
  sentinel, assert `DRAINING`, verify new admission is rejected, then release
  the client and assert `STOPPED` with exit code 0.
- A timeout test confirms an idle admitted connection is force-closed at the
  deadline and the process still emits `STOPPED`.
- Existing S7 stdin/stdout, S8 serial TCP, and S9 multi-client/backpressure
  tests remain green.
- Full Rust/Python suites, formatting, and evidence checks remain green.

## Acceptance and non-goals

S10 is accepted when a local supervisor can observe readiness, request a
controlled shutdown, see new admissions rejected, and obtain bounded drain
completion with deterministic exit behavior. It does not add native service
registration, OS signal adapters, TLS, authentication, persistence, broker,
remote deployment, or P0/performance qualification. The 100,000-frame target
remains reserved for benchmark decisions.
