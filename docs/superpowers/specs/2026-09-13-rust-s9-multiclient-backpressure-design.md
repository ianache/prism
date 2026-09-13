# Rust S9 Multi-Client Concurrency and Backpressure Design

**Status:** Proposed for review  
**Date:** 2026-09-13  
**Scope:** Extend the verified S8 TCP slice to bounded multi-client processing.

## Goal

Allow multiple local TCP clients to use `prism-run` concurrently through a
bounded worker pool, while preserving S7/S8 protocol semantics, per-connection
ordering, and an explicit capacity boundary.

## Relationship to S8

S8 remains the serial TCP mode and stdin/stdout behavior remains unchanged. S9
adds optional concurrency to the same binary and reuses the shared S7 line
processor. B0/B1/B2 semantics, B2 observer output, S5/S6 evidence, and the
100,000-frame benchmark target are unchanged.

## Chosen interface

```text
prism-run --route <b0|b1|b2> --listen 127.0.0.1:9000 \
  --workers <N> --connection-queue <M>
```

`--workers` defaults to `1`, preserving S8 behavior. `--connection-queue`
defaults to `0`, meaning no waiting connections beyond active workers. Both
values must be positive/nonnegative within a safe implementation limit and are
invalid in stdin/stdout mode. S9 remains loopback-oriented in documentation.

## Concurrency and ordering

- The listener accepts connections and submits them to a bounded queue.
- Exactly `N` worker threads process connections; each worker processes one
  connection serially and preserves request order within that connection.
- Different connections may interleave responses and have independent request
  sequence numbers.
- The S7 `process_line` function remains the only request dispatch boundary.
- The listener does not create an unbounded thread per client.

## Backpressure and rejection

The maximum admitted work is `workers + connection_queue`. If the queue is
full, the accepted socket is rejected immediately with one control JSONL error
line using route and code `CAPACITY_EXCEEDED`, then closed. This rejection is a
connection-level event and is not counted as a runtime frame outcome. The
listener remains alive and can accept later connections.

Workers block on socket I/O and process lines serially. A slow client consumes
one worker, making the bounded capacity visible and testable. No retry,
priority, dropping of already-admitted requests, or durable buffering is
introduced.

## Lifecycle

- Clean client EOF ends that connection and frees its worker.
- A later client can use the freed capacity.
- A malformed line remains an in-band S8 error and does not terminate the
  connection.
- An oversized line retains S8 behavior: emit `LINE_TOO_LARGE` and close only
  that connection.
- Worker or listener failures are reported as process/connection failures; the
  process must not silently claim successful processing.
- Shutdown behavior remains process termination by the host (for example,
  Ctrl+C); graceful drain is deferred.

## Verification

- Unit contracts cover worker/queue argument validation and capacity math.
- Process tests launch S9 with multiple workers, connect several clients, and
  verify concurrent handling, per-client ordering, client isolation, B0/B1/B2
  outcomes, and B2 observations.
- A deterministic saturation test holds workers busy, fills the queue, and
  verifies `CAPACITY_EXCEEDED` without killing the listener.
- An EOF/recovery test confirms capacity becomes available after a client
  closes.
- Existing S7 stdin/stdout and S8 serial TCP tests remain green.
- Full Rust release tests, Python tests, formatting, and evidence checks pass.

## Acceptance and non-goals

S9 is accepted when bounded multi-client TCP processing is reproducible locally,
capacity rejection is explicit and recoverable, and per-connection ordering is
preserved. It does not add a broker, persistent queue, TLS, authentication,
remote deployment, graceful drain, autoscaling, or P0/performance
qualification. The 100,000-frame target remains reserved for benchmark
decisions.
