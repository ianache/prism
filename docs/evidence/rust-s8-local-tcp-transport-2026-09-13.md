# S8 Local TCP Transport — Evidence

**Date:** 2026-09-13  
**Status:** local transport slice verified; not a deployment or P0 qualification

## Implemented boundary

`prism-run --route <b0|b1|b2> --listen 127.0.0.1:0` binds a loopback TCP
listener, announces the effective address on stderr, and processes multiple
JSONL envelopes per connection. It accepts another connection after clean EOF.
The S7 stdin/stdout mode remains available through the same shared
`process_line` boundary.

## Evidence

- Rust focused runner tests: 12 passed (including CLI, protocol, dispatch, and transport contracts).
- Rust workspace release suite: 57 tests passed, 0 failed.
- Python suite: 39 tests passed, 0 failed.
- `cargo fmt --package prism-runner -- --check`: passed.
- `git diff --check`: passed.
- TCP process tests covered B0, B1, and B2 with two valid envelopes and one malformed hex envelope on a persistent connection, followed by a second client connection.
- B2 returned six observer events for valid frames; malformed input returned `INVALID_HEX` in band and did not break the connection.
- An oversized input line was rejected with `LINE_TOO_LARGE`, the current connection closed, and the listener process remained alive.

The tests used an existing fixture from `tests/fixtures/p0-smoke`; no binary
dataset or generated result was added to Git.

## Boundaries

S8 uses one active client connection at a time and loopback TCP only. It does
not add multi-client concurrency, TLS, authentication, retries, durable queues,
brokers, supervision, network deployment, or performance qualification. The
100,000-frame target remains reserved for benchmark decisions. S5/S6 evidence,
including the unresolved p99 findings, is unchanged.
