# Rust S13 Process Supervision and Release Hardening

**Status:** approved for implementation
**Scope:** make the secure local `prism-run` flow operable through reproducible
PowerShell commands without adding a distributed runtime.

## Goal

Provide a portable operator contract to start, check, stop, and verify the
TLS/authenticated runner process.

## Design

Add a PowerShell supervisor that owns a temporary runtime directory, process
ID, stderr state file, and shutdown sentinel. `start` launches the release
binary and waits for `READY`; `check` verifies the process is alive and can
complete a TLS/authenticated JSONL request; `stop` creates the sentinel and
waits for `STOPPED`, terminating only when the controlled process fails to
exit. Existing runner behavior and CLI flags remain unchanged.

## Constraints and non-goals

- Preserve the 100,000-frame target and S7–S12 contracts.
- Keep loopback/local operation and explicit TLS/token configuration.
- Never print or persist token contents, private keys, or certificate data.
- No Windows service registration, broker, persistence, Kubernetes, mTLS,
  rotation, remote deployment, or P0 qualification.

## Acceptance criteria

1. `start` reports a usable address and writes only non-secret process state.
2. `check` validates process liveness, TLS, authentication, and JSONL response.
3. `stop` requests drain and verifies `STOPPED` with bounded waiting.
4. A failed/stale PID state is detected without killing unrelated processes.
5. Manual operator flow and automated regression tests are documented.
