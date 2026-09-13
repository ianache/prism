# S7 Production Vertical Slice — Evidence

**Date:** 2026-09-13  
**Scope:** local stdin → Rust runtime → stdout flow  
**Status:** functional slice verified; not a network service or P0 qualification

## Contract exercised

`prism-run --route b0|b1|b2 [--request-id-prefix <prefix>]` reads JSONL
envelopes of the form:

```json
{"request_id":"one","payload_hex":"<canonical frame bytes in hex>"}
```

The command emits one JSON object per input line, flushes each result, keeps
input order, reports malformed lines in band, and exits cleanly at EOF.

## Verification

- `cargo test --workspace --release`: passed after adding `prism-runner`; existing runtime and benchmark contracts remained green.
- `cargo fmt --all -- --check`: passed.
- `python -m unittest tests/test_s7_vertical_slice.py`: 1 test passed.
- `python -m unittest discover -s tests`: 37 tests passed.
- Process E2E covered B0, B1, and B2 with two valid fixture lines plus one invalid hex line per route: 3 output rows per route, ordered IDs, invalid input returned as `INVALID_HEX`, and B2 reported 6 observer events for valid frames.

The representative fixture came from the existing `tests/fixtures/p0-smoke`
corpus and no generated dataset or binary fixture was added to Git.

## Boundary and non-goals

Runtime rejections remain valid outcomes and adapter/parser errors use
`ok:false` with stable codes. B2 reuses B1’s pipeline and observer hook; it
does not alter outcome semantics. S5/S6 evidence and their negative p99
findings are unchanged. S7 does not claim network readiness, broker support,
100K performance evidence, or P0 production qualification.
