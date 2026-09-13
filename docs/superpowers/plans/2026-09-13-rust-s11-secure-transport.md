# Rust S11 Secure Transport and Authentication — Implementation Plan

> **Execution note:** Implement task-by-task with verification after each task.

**Goal:** Add optional TLS and token authentication to `prism-run` TCP mode while preserving S7 JSONL, S9 bounded concurrency, and S10 lifecycle behavior.

**Design:** [S11 secure transport](../specs/2026-09-13-rust-s11-secure-transport-design.md)

## Invariants

- TLS is optional only for the existing local plaintext mode; no silent fallback when TLS is configured.
- Certificate/key paths and token file are validated before `READY`.
- Tokens are never logged, echoed, passed through CLI/environment, or included in responses.
- TLS handshake happens before S9 worker admission.
- Reuse `process_line` and preserve B0/B1/B2 outcomes, B2 observations, ordering, capacity, and S10 shutdown.
- Do not modify benchmark paths, S5/S6 evidence, generated datasets, or the 100,000-frame benchmark target.
- No certificate rotation, mTLS, broker, persistence, remote deployment, or P0 claim.

## Task 1: Add secure CLI and configuration contracts

**Files:** `crates/prism-runner/src/cli.rs`, `crates/prism-runner/src/security.rs`, `crates/prism-runner/tests/route_contract.rs`, `crates/prism-runner/tests/security_contract.rs`

1. Add failing tests for `--tls-cert`, `--tls-key`, `--auth-token-file`, pairing rules, missing files, empty token files, and invalid use outside TCP mode.
2. Add typed security configuration and deterministic startup errors.
3. Preserve existing route, listen, worker, queue, sentinel, and timeout defaults.
4. Define token comparison and redaction helpers without exposing credential contents.
5. Run focused tests and package formatting checks.

## Task 2: Add TLS configuration and handshake boundary

**Files:** `Cargo.toml`, `Cargo.lock`, `crates/prism-runner/Cargo.toml`, `crates/prism-runner/src/tls.rs`, `crates/prism-runner/src/transport.rs`, `crates/prism-runner/tests/tls_contract.rs`

1. Select and pin an actively maintained Rust TLS implementation using its official API documentation; keep the dependency limited to the runner.
2. Add failing tests for PEM loading, certificate/key mismatch, handshake success, and handshake failure isolation.
3. Load and validate TLS server configuration before listener readiness.
4. Perform the TLS handshake immediately after accept and before queue/capacity admission.
5. Close failed handshakes without terminating the listener and without counting them as runtime frames.
6. Keep plaintext transport path unchanged when TLS is not configured.

## Task 3: Integrate per-line token authentication

**Files:** `crates/prism-runner/src/protocol.rs`, `crates/prism-runner/src/output.rs`, `crates/prism-runner/src/transport.rs`, `crates/prism-runner/tests/security_contract.rs`

1. Add failing tests for missing, incorrect, and correct `auth_token` values, including token redaction in serialized errors.
2. Extend the envelope parser to expose an optional authentication token without weakening required `request_id`/`payload_hex` validation.
3. Require and compare the token only when authentication is configured; otherwise preserve S7/S8 behavior.
4. Reject unauthorized lines with `AUTHENTICATION_FAILED` before runtime dispatch while keeping the connection usable.
5. Ensure token values never appear in stderr, stdout responses, panic text, or test diagnostics.

## Task 4: Add secure process E2E and lifecycle regression tests

**Files:** `tests/test_s11_secure_transport.py`, `docs/rust-b0-b1.md`, `docs/evidence/rust-s11-secure-transport-2026-09-13.md`

1. Add test certificate material generated in-memory or test-only fixtures; do not commit production secrets.
2. Launch TLS mode for B0, B1, and B2, connect with a trusted TLS client, and verify ordered JSONL outcomes.
3. Verify authorized requests, missing/incorrect tokens, handshake failure, B2 observations, and listener survival.
4. Run secure mode with S9 workers/queue and S10 sentinel shutdown; verify readiness, draining, and stopped states.
5. Rerun unchanged plaintext S7–S10 tests.
6. Document local certificate/token setup and explicit security limitations.

## Task 5: Final audit, evidence, consumption record, and commit

**Files:** `docs/consumo.md`, `docs/evidence/rust-s11-secure-transport-2026-09-13.md`

1. Run the complete Cargo release workspace suite and all Python tests.
2. Run runner formatting validation and `git diff --check`.
3. Review that no private key, token, generated certificate, benchmark file, or S5/S6 evidence changed.
4. Record TLS/auth configuration shape, authorized/unauthorized counts, handshake isolation, lifecycle checks, and test results without recording secrets.
5. Append one consumption row per S11 task and one plan-completion row, using real measurements when available and `N/D` otherwise.
6. Commit only after verification succeeds; do not claim remote deployment, mTLS, or P0 readiness.

## Verification commands

```powershell
rtk proxy 'C:\Users\ilver\.cargo\bin\cargo.exe' test --workspace --release
rtk proxy python -m unittest discover -s tests
rtk proxy 'C:\Users\ilver\.cargo\bin\cargo.exe' fmt --package prism-runner -- --check
rtk git diff --check
```

## Completion criteria

- TLS-configured TCP mode validates credentials before readiness and has no plaintext fallback.
- Authenticated B0/B1/B2 JSONL processing works through the existing S7–S10 path.
- Unauthorized requests and failed handshakes are isolated and non-secret-bearing.
- Plaintext regression suites and full workspace verification remain green.
- Evidence and consumption records are complete with explicit non-goals.
