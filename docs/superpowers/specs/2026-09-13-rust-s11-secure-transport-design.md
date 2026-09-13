# Rust S11 Secure Transport and Authentication Design

**Status:** Proposed for review  
**Date:** 2026-09-13  
**Scope:** Add optional TLS and transport-level client authentication to the local TCP slice.

## Goal

Provide a secure TCP mode for the S10 lifecycle runner without changing the
JSONL framing, B0/B1/B2 runtime semantics, S9 capacity rules, or S10 shutdown
behavior.

## Relationship to S10

S10 remains the lifecycle and drain controller. S11 wraps each accepted TCP
stream with TLS before handing it to the existing bounded connection worker.
The plaintext loopback mode remains available for local development when TLS
is not configured. S5/S6 evidence and the 100,000-frame benchmark target are
unchanged.

## Chosen interface

Secure TCP mode requires a certificate and private key together:

```text
prism-run --route <b0|b1|b2> --listen 127.0.0.1:9000 \
  --tls-cert <certificate.pem> --tls-key <private-key.pem> \
  [--auth-token-file <token-file>]
```

The token file is optional for TLS-only operation but, when configured, every
JSONL envelope must include an `auth_token` string matching the external token.
The token is checked before `payload_hex` dispatch, never echoed, logged, or
included in an error response. Missing or incorrect tokens produce an in-band
`AUTHENTICATION_FAILED` response and do not terminate the connection.

TLS arguments are invalid unless `--listen` is used. Cert/key must be supplied
together. The default plaintext mode remains loopback-oriented and is not
described as suitable for remote deployment.

## Security semantics

- TLS handshake completes before a connection is admitted to the S9 worker pool.
- Invalid certificates, malformed keys, handshake failure, and TLS I/O errors
  reject/close only the affected connection and do not become runtime outcomes.
- Certificate and key paths are validated at startup before `READY`.
- Token file contents are read once at startup, trimmed only for one final line
  ending, and held in memory without appearing in diagnostics.
- Empty token files and mismatched cert/key configuration are startup errors.
- No plaintext fallback occurs for a TLS-configured listener.
- No token is accepted from a command-line argument or environment variable.

## Protocol compatibility

The existing JSONL envelope remains:

```json
{"request_id":"one","payload_hex":"...","auth_token":"..."}
```

`auth_token` is required only when authentication is enabled. The response
shape remains S7/S8 compatible (`request_id`, `ok`, `route`, and outcome/error)
and does not include credentials. Per-connection ordering, S9 capacity and
S10 lifecycle states remain unchanged.

## Architecture

```text
TCP accept → TLS handshake → bounded admission → token check per line
           → S7 parser/dispatch → S9 worker → S10 lifecycle response
```

The secure transport owns TLS configuration, handshake, credential loading,
and authentication. The existing runner owns line bounds, lifecycle, capacity,
request processing, and response serialization. Benchmark code remains
separate and does not use the secure transport path.

## Verification

- Unit contracts cover cert/key pairing, token-file validation, redaction, and
  authentication decisions.
- Process tests launch TLS mode with test certificates, connect with a trusted
  client, verify B0/B1/B2 results, and verify incorrect/missing tokens.
- A handshake failure test confirms the listener remains available.
- Existing plaintext S7–S10 tests remain green.
- Lifecycle shutdown, backpressure, line limits, and per-connection ordering
  remain covered over secure streams.
- Full Rust/Python suites, formatting, and evidence checks pass.

## Acceptance and non-goals

S11 is accepted when a local TLS client can authenticate and execute the same
JSONL B0/B1/B2 flow, unauthorized requests are rejected without secret
disclosure, and S7–S10 behavior remains green. It does not add certificate
rotation, an external identity provider, mTLS policy, remote deployment,
broker, persistence, service registration, or P0/performance qualification.
The 100,000-frame target remains reserved for benchmark decisions.
