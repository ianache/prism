# Telemetry Specification Gap and Ambiguity Report

**Status:** Resolved for specification release; runtime implementation gate remains closed until REVIEW.md records zero unresolved blockers.

| ID | Topic | Current statement | Ambiguity | Impact | Proposed resolution | Status | Owner |
| --- | --- | --- | --- | --- | --- | --- | --- |
| FRAME-001 | Frame widths/offsets | Fixed little-endian frame is required | Exact wire layout absent | Bytes could differ | v1 table in telemetry-workflow.md; 41-byte prefix, 6-byte records, lengths 105/249/501 | RESOLVED | Telemetry owner |
| FRAME-002 | Checksum | F1 validates terminal checksum | Algorithm absent | Valid bytes could differ | CRC-32C Castagnoli, reflected 0x82F63B78, init/xor 0xFFFFFFFF, offsets 0..checksum-1 | RESOLVED | Telemetry owner |
| FRAME-003 | Timestamp | Timestamp is normalized | Epoch/unit absent | Outputs could differ | uint64 Unix seconds UTC, range 0..4102444800 | RESOLVED | Telemetry owner |
| FRAME-004 | Coordinates | Fixed-point position required | Scale absent | Position bytes could differ | signed int32 degrees x 10^7, exact integer division | RESOLVED | Telemetry owner |
| CONTRACT-001 | Schema versions | JSON Schema load-time validation | Compatibility absent | Contracts could load differently | Draft 2020-12, schema_version 1.0, exact-major, local refs | RESOLVED | Contract owner |
| RULE-001 | Rule table | F4/F5/F6 deterministic | Thresholds and conflicts absent | Outputs could differ | versioned ordered rule-table.json, first-match inclusive semantics | RESOLVED | Domain owner |
| ERROR-001 | Error taxonomy | F1 typed rejection categories required | Codes/precedence absent | Rejections could differ | seven stable codes and fixed validation order in telemetry-workflow.md | RESOLVED | Contract owner |
| DATA-001 | Fixture encoding | Seeded 100/250/500-byte classes | PRNG/manifest absent | Inputs could differ | SplitMix64 seed 0x505249534D5F5631, raw bin plus canonical JSONL and SHA-256 | RESOLVED | Benchmark owner |
| BENCH-001 | Warm-up/sampling | Warmed matched runs required | Percentile/tax rules absent | Measurements could differ | 10k warm-up, 5x1M repetitions, nearest-rank, monotonic ns, no outliers | RESOLVED | Methodology owner |
| BENCH-002 | Metadata | Hardware/runtime metadata required | Format/missing-data absent | Runs could not be audited | required machine-readable fields; missing required fields invalidate run | RESOLVED | Methodology owner |
| QOS-001 | Gate interpretation | P0/P1 thresholds are provisional | bounded/material undefined | Qualification could differ | <10% RSS growth and <=10% S5 p99 degradation; all repetitions must pass | RESOLVED | Performance owner |

## Resolution record

These values are authorized v1 specification decisions for the open details in the approved design. A future change must increment the relevant version and regenerate fixtures; it may not reinterpret v1 bytes silently.

## Non-blocking compatibility

P2-P4 adapters, schema evolution, durable execution, and agentic replay remain outside this freeze and are NON-BLOCKING.
