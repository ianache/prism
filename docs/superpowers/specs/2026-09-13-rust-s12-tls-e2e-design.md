# Rust S12 TLS E2E Stabilization Design

**Status:** approved for implementation
**Scope:** complete the missing process-level TLS verification for the local
TCP slice introduced by S11.

## Goal

Demonstrate that a real TLS client can connect to `prism-run`, exchange the
existing JSONL envelopes for B0/B1/B2, authenticate each line, survive invalid
credentials, and preserve S9 capacity plus S10 lifecycle behavior.

## Design

Keep Rustls as the server implementation and keep the existing plaintext path
unchanged. The test harness will use a deterministic test-only certificate
chain and a client implementation whose Windows socket behavior is explicit;
no production certificate, key, or token is committed. The server performs
the blocking TLS handshake after `accept` and before capacity admission, with
an isolated handshake timeout. After a successful handshake, the existing
bounded worker and per-line authentication flow handles JSONL.

The test must verify both TLS transport and application behavior: ordered
responses, missing/incorrect/correct tokens, B2 observations, handshake
failure isolation, capacity rejection/recovery, and controlled draining.

## Constraints and non-goals

- Preserve the 100,000-frame benchmark target and all S7–S11 contracts.
- Keep TLS optional and require certificate/key pairing; never silently fall
  back to plaintext when TLS is configured.
- Never log, echo, or persist credentials or private key material.
- No mTLS, certificate rotation, broker, persistence, remote deployment, or
  P0 qualification.

## Acceptance criteria

1. A process-level TLS client completes handshake and exchanges JSONL.
2. B0, B1, and B2 produce the existing canonical responses over TLS.
3. Missing or incorrect authentication returns `AUTHENTICATION_FAILED` and
   the same connection accepts a subsequent authorized line.
4. A failed handshake does not consume worker capacity or stop the listener.
5. S9 capacity and S10 `DRAINING`/`STOPPED` behavior remain correct in secure
   mode.
6. Full Cargo and Python suites, manual verification, evidence, and the
   consumption log are updated.
