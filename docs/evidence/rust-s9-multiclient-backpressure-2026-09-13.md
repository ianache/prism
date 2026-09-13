# S9 Multi-Client Concurrency and Backpressure — Evidence

**Date:** 2026-09-13  
**Status:** bounded local TCP slice verified; not a deployment or P0 qualification

## Implemented boundary

`prism-run --route <b0|b1|b2> --listen 127.0.0.1:0 --workers N
--connection-queue M` uses a fixed worker pool and explicit admission capacity
of `N + M`. S7 stdin/stdout and S8 serial TCP remain available and reuse the
same `process_line` boundary.

## Verification

- Rust workspace release suite: 58 tests passed, 0 failed.
- Python suite: 41 tests passed, 0 failed.
- Runner focused contracts: 13 tests passed, 0 failed.
- Runner formatting check: passed.
- `git diff --check`: passed.
- Multi-client test used 2 workers and queue capacity 2; three clients were
  admitted, each preserved its own request order, and B2 emitted six observer
  events per valid request.
- Saturation test used 1 worker and queue capacity 0; a held client caused the
  next connection to receive `CAPACITY_EXCEEDED`, while the listener remained
  alive and admitted a client after EOF freed capacity.

Existing S7/S8 tests remained green. Tests used the existing p0-smoke fixture;
no generated dataset or binary fixture was added to Git.

## Boundaries

S9 provides bounded local multi-client processing only. It does not add a
broker, durable queue, TLS, authentication, graceful drain, autoscaling,
remote deployment, or performance qualification. The 100,000-frame target
remains reserved for benchmark decisions. S5/S6 evidence and unresolved p99
findings are unchanged.
