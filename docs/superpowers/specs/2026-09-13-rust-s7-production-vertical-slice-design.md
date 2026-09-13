# Rust S7 Production Vertical Slice Design

**Status:** Proposed for review  
**Date:** 2026-09-13  
**Scope:** A real, continuously executable local flow from framed input through the Rust runtime to contractual output.

## Goal

Provide a supported end-to-end command that reads one JSONL telemetry frame
per line from `stdin`, processes it through the selected B0/B1/B2 route, and
writes one JSONL result per input line to `stdout`.

## Current boundary

`prism-runtime` already implements frame decoding, frozen rule precedence, B0,
B1, and B2 observed processing. `prism-bench` drives those components with
static datasets and measures them, but it is not a live ingress/egress flow.
S7 adds an operational adapter without changing the runtime semantics or the
S5/S6 measurement paths.

## Chosen interface

The first production slice uses line-delimited JSON because it is inspectable,
scriptable, and does not introduce a network dependency. Each input line is a
complete protocol frame encoded as JSON with the existing model fields. Each
output line contains the normalized outcome, route, rejection context, and a
stable request identifier. Blank input lines are rejected with a structured
error result rather than silently skipped.

The command accepts an explicit route (`b0`, `b1`, or `b2`), reads until EOF,
flushes each result, returns exit code 0 when all lines are processed, and
returns a nonzero exit code only for process-level failures such as invalid
startup configuration or stdout failure. A malformed individual frame is
represented in-band and does not terminate the stream.

## Architecture and data flow

`stdin` → line reader → JSON/frame decoder → selected runtime route → result
serializer → `stdout`. The adapter owns buffering, request IDs, structured
errors, and flushing. `prism-runtime` owns protocol validation, filter
precedence, classification, and normalized outcome semantics. No benchmark
timer, p99 metric, workflow tax, broker, queue, or transport abstraction is
added to the live path.

## CLI and lifecycle

Add a dedicated binary command, `prism-run`, with required `--route` and
optional `--request-id-prefix`. It supports `--help`, rejects unknown routes,
handles EOF cleanly, propagates broken-pipe errors, and never writes temporary
or partial output files. The existing benchmark CLI remains unchanged.

## Output contract

Every input line produces exactly one output line containing `request_id`,
`ok`, `route`, and either `outcome` or a structured `error` object. Runtime
rejections are valid results (`ok: true` with rejection details); decode and
validation failures are `ok: false` with stable error codes. Output is flushed
after each line so a downstream process can consume results incrementally.

## Verification and acceptance

- Unit contracts cover route selection, request IDs, EOF, blank/malformed lines, runtime rejections, and JSON serialization.
- End-to-end tests launch `prism-run`, feed multiple frames through stdin, parse stdout, and verify one result per input in order.
- B0/B1/B2 results match the existing runtime contracts and Python smoke oracle where applicable.
- Cargo workspace release tests and all Python tests remain green.
- A documented local command executes the flow with a reproducible smoke fixture.
- S5/S6 evidence remains immutable and is not requalified by S7.
- S7 is supported as a local streaming vertical slice, not yet as a network service or P0 production deployment.
