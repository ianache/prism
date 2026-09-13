# Rust S1 Evidence Completeness Design

**Status:** Proposed
**Date:** 2026-09-12
**Scope:** Follow-up to the integrated Rust B0/B1 runtime and protocol S1/100K validation.

## Goal

Make every Rust S1 JSONL record self-describing and independently auditable
against the frozen benchmark protocol, without changing B0/B1 processing or
adding new benchmark levels.

## Current gap

The current runner produces correct 300-fixture and 100K evidence, but the raw
record does not expose all protocol inputs and outcomes. In particular, the
record lacks dataset ID and workload counts, timestamp, warm-up/convergence
parameters and result, sample count per repetition, typed rejection and
execution-failure counts, and explicit unavailable resource metrics. The
runner also reports one aggregate correctness total on every repetition rather
than a clearly scoped per-repetition record.

## Design

Extend the existing standard-library-only boundaries:

- `Dataset` exposes stable dataset ID, fixture count, payload-class totals and
  validity-category totals already read from the manifest.
- `RawRun` carries warm-up target/actual frames, convergence window and result,
  measured-frame count, typed rejections, execution failures, and per-run
  correctness totals.
- `RawRecord` serializes those values plus UTC timestamp, commit, exact command,
  dataset identity, and all required metadata in deterministic key order.
- `Metadata::collect` receives the complete invocation context and records
  unavailable host values as `N/D`; it never executes host inspection inside
  the timed closure.
- The Python audit validates required keys, numeric domains, digest and dataset
  identity, five repetitions per level, correctness, convergence, and the
  workflow-tax formula. It rejects incomplete or inconsistent raw evidence.

The primary timer remains limited to resident frame processing. Correctness
comparison, outcome classification, percentile aggregation, serialization and
writing remain outside it. Existing write-once atomic behavior is preserved.

## Acceptance criteria

1. Every S1 raw record contains dataset identity/counts, invocation identity,
   protocol parameters, convergence evidence, complete timing metrics,
   correctness/failure counts, host metadata, and workflow tax.
2. A 300-fixture smoke run and the external 100K run produce five valid records
   for each of B0 and B1; each record has internally consistent counts.
3. The audit rejects missing fields, mismatched counts/digests, invalid tax,
   non-finite metrics, wrong repetition cardinality, and failed convergence.
4. `cargo test --workspace --release` remains green and Rust runtime behavior
   is unchanged.
5. No 100K binary fixtures are committed, and no P0, B2/B3, S2–S5, broker,
   tracing, transport, or production-workflow claim is introduced.

## Non-goals

- New runtime behavior or rule-table changes.
- B2/B3 implementation or S2–S5 execution.
- Resource telemetry that is unavailable on the host; such values remain `N/D`.
- Re-running or committing the 1,000,000-file release corpus.
