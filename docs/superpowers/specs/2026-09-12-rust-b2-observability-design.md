# Rust B2 Observability and 100K Decision Baseline

**Status:** Proposed
**Date:** 2026-09-12
**Scope:** Next increment after Rust B0/B1 and complete S1 evidence.

## Goal

Measure the cost of local observability on the Rust PRISM pipeline while
establishing 100,000 measured frames per repetition as the normative baseline
for all subsequent benchmark decisions.

## Decisions

1. B2 is B1 with local structured observability added around filters F1–F6.
2. The first B2 variant uses no broker, network, persistence, OpenTelemetry
   exporter, or external telemetry service.
3. B2 records one execution ID, per-filter timing, invocation counters,
   rejection/failure counters, and aggregate correctness for each repetition.
4. Observability instrumentation is inside the primary timer so the measured
   difference represents the runtime cost of B2 observability.
5. The comparison uses `Observability Tax = (B2 p99 - B1 p99) / B1 p99 * 100`,
   matched by repetition. Negative values are retained.
6. Every subsequent comparable decision uses 100,000 measured frames per
   repetition, five repetitions, 10,000 warm-up frames, and 1,000-frame
   convergence windows. A different frame target requires a new versioned
   protocol decision.

## Protocol change

The previous S2 target of 1,000,000 measured frames is replaced by 100,000
measured frames. The external 100K corpus is the official decision corpus for
B2 and later comparable benchmark decisions. Existing 10,000-frame S1 results
remain historical evidence and are not retroactively reinterpreted.

The protocol version must be incremented for this change. The updated protocol
must state explicitly that the 100,000-frame target applies to B2 and all
future comparable decisions, including later runtime, serialization,
observability, and transport comparisons. Scenario-specific exceptions are
not permitted unless separately approved and versioned.

## Architecture

The existing `prism-runtime` filter behavior remains unchanged. B2 adds a
runtime-neutral `Observer` interface in `prism-runtime`; `Pipeline::process`
continues to use a no-op observer, while B2 calls an observed processing entry
point with a local collector. This keeps B0 and ordinary B1 runs free of B2
collection and gives the benchmark a stable boundary for per-filter evidence.

For each measured frame, the B2 path creates or receives an execution ID,
records monotonic timestamps around F1–F6 through the observer callbacks,
updates counters, and returns the same logical outcome as B1. The observer
must expose a fixed six-filter key set even when a later filter is not reached;
unreached filters have zero invocations and zero timings for that frame.
Correctness comparison, percentile aggregation, workflow-tax calculation,
serialization, and file writing remain outside the primary timer except for
the instrumentation work explicitly being measured.

The first implementation must use Rust standard-library facilities only. It
must not introduce threads, locks, allocation-heavy event pipelines, network
clients, brokers, exporters, or persistent trace storage.

## Evidence model

Each B2 raw record must include the existing self-describing S1 fields plus:

- `observability_variant` with the value `local_metrics`;
- `execution_id` or a deterministic execution identity scoped to the record;
- `filter_timings_ns` for F1 through F6;
- `filter_invocations` for F1 through F6;
- `filter_rejections` and `filter_execution_failures` for F1 through F6;
- `observability_tax_percent` matched to the B1 repetition;
- `measured_frames = 100000` and the protocol version.

The audit must reject missing filter keys, mismatched invocation totals,
negative or non-finite timing values, incorrect repetition pairing, incorrect
tax arithmetic, incorrect frame targets, and correctness mismatches.

## Correctness and acceptance

B2 must produce exactly the same normalized, rejected, and failed logical
outcomes as B1 for every fixture. Correctness is a hard prerequisite for any
performance conclusion.

The increment is accepted only when:

- the 100K B1 baseline and 100K B2 records each contain five valid repetitions;
- all records pass independent JSONL audit;
- B2 filter counters sum consistently with measured frames and outcomes;
- B2 p50, p95, p99, p99.9, max, throughput, and observability tax are present;
- B0/B1 behavior and existing S1 evidence contracts remain green;
- no 1,000,000-frame claim is made for the new decision baseline;
- no external observability system or transport is introduced.

## Non-goals

- OpenTelemetry export or distributed tracing.
- Broker, network, persistence, or durable execution.
- B3 transport evaluation.
- S3 concurrency, S4 burst, or S5 sustained-load qualification.
- Reinterpretation of historical 10K S1 measurements.
- P0 qualification solely from this increment.

## Risks and mitigations

Instrumentation can alter allocation behavior, branch prediction, and cache
pressure. The raw record must expose the selected variant and frame target,
and the report must call out these threats. B1 must remain a separately run
baseline with identical dataset, warm-up, repetition count, and measured-frame
target so the tax comparison is matched and auditable.
