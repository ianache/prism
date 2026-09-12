# P0 Telemetry Workflow Specification Map

**Status:** Specification skeleton; implementation gate closed pending the [gap and ambiguity report](gap-ambiguity-report.md).

## Purpose and scope

This maps the approved P0 telemetry workflow into documents that will freeze byte, contract, and deterministic-result semantics. It is specification-only: it does not define a runtime, generator, broker, or benchmark harness.

The mandatory ordered P0 path is `resident BinaryFrame -> F1 Validate -> F2 Decode -> F3 Normalize -> F4 Evaluate Rules -> F5 Classify -> F6 Route -> NormalizedTelemetry or Rejection`.

Correctness against the independent oracle is a hard prerequisite for performance qualification. No runtime implementation may begin while a `BLOCKING` decision in the [gap report](gap-ambiguity-report.md#blocking-decision-register) is unresolved.

## Frame semantics

The future canonical frame is `Header -> Core telemetry fields -> Bounded sensor/IO area -> Checksum`. It includes magic, version, flags, payload length, protocol, device identifier, timestamp, position, speed, heading, ignition, battery, bounded sensor/IO data, and checksum. All multi-byte values are little-endian. Numeric canonical correctness uses declared integer or fixed-point units, not floating-point arithmetic.

Exact field offsets, widths, checksum coverage, timestamp epoch/unit, and coordinate scales remain `BLOCKING` in `FRAME-*` rows of the [gap report](gap-ambiguity-report.md#blocking-decision-register). Resulting `BinaryFrame` and `ValidatedFrame` schemas belong under `contracts/data/` in the next task.

## Data semantics

The canonical data contracts are `BinaryFrame`, `ValidatedFrame`, `NormalizedTelemetry`, and `Rejection`. They will be canonical JSON documents validated with JSON Schema at control-plane/load time, never per filter invocation. Expected outcomes compare logical values, not language-specific object layouts or incidental metadata.

Fixture encoding, deterministic seeds, and the independent oracle are `BLOCKING` as `DATA-001` in the [gap report](gap-ambiguity-report.md#blocking-decision-register). Workload composition remains 80% valid, 5% invalid, and 15% edge/complex across approximately 100-, 250-, and 500-byte payload classes.

## Filter semantics

| Filter | Required transition |
| --- | --- |
| F1 Validate | Checks framing, version, length, ranges, supported protocol, and checksum; emits `ValidatedFrame` or `Rejection`. |
| F2 Decode | Converts validated wire data into typed telemetry fields. |
| F3 Normalize | Applies canonical units, timestamp and coordinate normalization, and declared sensor defaults. |
| F4 Evaluate Rules | Applies the frozen versioned rule set using integer/fixed-point comparisons. |
| F5 Classify | Maps rule results to deterministic classification and severity. |
| F6 Route | Maps classification and context to a deterministic route enum; performs no network or broker operation in P0. |

Malformed frames stop at F1 with a typed rejection. Unexpected runtime or filter faults are execution failures, not typed rejections. Stable errors and deterministic rules are `BLOCKING` as `ERROR-001` and `RULE-001` in the [gap report](gap-ambiguity-report.md#blocking-decision-register).

## Workflow semantics

The P0 workflow contract will declare identity, version, typed inputs and outputs, exactly ordered nodes F1-F6, and execution profile `P0`. Filter contracts declare identity, version, typed input/output, errors, and capabilities. Versioning and schema-loading policy are `BLOCKING` as `CONTRACT-001` in the [gap report](gap-ambiguity-report.md#blocking-decision-register).

Control-plane/load-time validation must reject incompatible workflow, filter, data, and QoS contracts before a timed invocation. The pipeline itself has no per-invocation contract-loading validation.

## QoS and error semantics

P0 is in-process with no broker, database, HTTP call, LLM, or human interaction in its hot path. It requires bounded memory, minimal allocations, and explicit streaming backpressure. Provisional P0 and P1 gates are mapped to their measurements in [acceptance criteria](acceptance-criteria.md); their interpretation is `BLOCKING` as `QOS-001`.

Typed rejections must carry stable error code and context; runtime/filter faults remain separate. The timer, scenarios, metrics, and reporting rules that make QoS measurements comparable are defined in [benchmark protocol](benchmark-protocol.md).

## Artifact relationships

| Concern | Governing artifact |
| --- | --- |
| Frame and data | This document; [gap report](gap-ambiguity-report.md); future `contracts/data/` |
| Filters, workflow, rules, errors | This document; [gap report](gap-ambiguity-report.md); future `contracts/filter/` and `contracts/workflow/` |
| QoS and acceptance | [acceptance criteria](acceptance-criteria.md) |
| Dataset and measurement | [benchmark protocol](benchmark-protocol.md) |
