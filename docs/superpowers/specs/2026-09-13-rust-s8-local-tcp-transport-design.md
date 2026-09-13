# Rust S8 Local TCP Transport Design

**Status:** Proposed for review  
**Date:** 2026-09-13  
**Scope:** Add a locally executable TCP boundary to the verified S7 vertical slice.

## Goal

Expose the existing `prism-run` JSONL processing flow over a local TCP socket,
so a client can send newline-delimited JSON envelopes and receive one response
per request without changing B0/B1/B2 runtime semantics.

## Relationship to S7

S7 remains the canonical adapter boundary and continues to support stdin/stdout.
S8 adds a transport mode around the same parser, route dispatch, observer, and
serializer. No benchmark path, S5/S6 evidence, or 100K performance conclusion
is changed.

## Chosen interface

The command accepts either:

```text
prism-run --route <b0|b1|b2>
prism-run --route <b0|b1|b2> --listen <host:port>
```

Without `--listen`, behavior is the existing stdin/stdout mode. With
`--listen`, the process binds a TCP listener and handles one client connection
at a time. A connection may carry multiple JSONL envelopes; each newline
delimits one request and each request receives one newline-delimited response.
The existing `request_id` and `payload_hex` envelope is unchanged.

The listener reports its bound address only through a controlled startup
message on stderr. It does not write protocol responses to stdout in TCP mode.
Port `0` is allowed for tests and causes the OS to select an available port.

## Connection and error semantics

- The listener accepts connections serially and processes lines serially.
- A client can keep a connection open for multiple requests.
- A clean client EOF closes that connection and returns to accept another one.
- Invalid JSONL/hex input produces an in-band S7 error response and the
  connection remains usable.
- A line exceeding the fixed 64 KiB maximum produces `LINE_TOO_LARGE` and the
  connection is closed to avoid ambiguous framing.
- Bind, accept, read, write, and flush failures are process/connection errors;
  they do not become runtime outcomes.
- There is no retry, durable queue, broker, TLS, authentication, or remote bind
  requirement in S8. The documented default bind is loopback.

## Architecture

```text
TCP listener → connection line reader → S7 envelope parser
             → hex decoder → selected B0/B1/B2 dispatch
             → S7 serializer → connection writer
```

The transport module owns socket lifecycle, line bounds, connection handling,
and startup errors. S7 owns request parsing, IDs, runtime dispatch, B2
observation, and result serialization. The stdin/stdout path should call the
same shared line-processing function so the two modes cannot diverge.

## Verification

- Unit contracts cover listen argument parsing, address parsing, line limits,
  and mode selection.
- Process-level TCP tests bind an ephemeral loopback port, connect a client,
  send multiple valid and malformed envelopes, verify ordered one-for-one
  responses for B0/B1/B2, and verify the connection remains usable after an
  in-band error.
- A lifecycle test verifies clean client EOF and a second accepted connection.
- Existing S7 stdin/stdout E2E remains green.
- Full Cargo release tests, Python tests, and formatting checks remain green.
- Evidence records the command, loopback address, fixture identity, request and
  response counts, exit behavior, and explicit non-goals.

## Acceptance and non-goals

S8 is accepted when a documented localhost TCP client can execute B0/B1/B2
requests through the same verified runtime path, malformed input is safely
handled, and the complete existing suite remains green. This is a local
transport slice, not a production deployment or P0 qualification. Multi-client
concurrency, TLS/authentication, backpressure queues, broker integration,
process supervision, and performance qualification remain later increments.
