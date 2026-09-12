# Telemetry Benchmark Specification Freeze Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Convert the approved telemetry-spike design into an unambiguous, machine-readable benchmark specification that can be implemented identically in Rust, Java, and Go.

**Architecture:** Keep the first deliverable specification-only: canonical binary-frame and typed-contract definitions live under `specification/` and `contracts/`, while deterministic fixture generation and the correctness oracle are separate, language-neutral artifacts under `datasets/`. Runtime implementations, brokers, UI, and deployment infrastructure are explicitly deferred until the ambiguity report is resolved.

**Tech Stack:** Markdown, JSON Schema Draft 2020-12, JSON/JSONL fixtures, deterministic generator/oracle implementation selected during planning, and reproducible command documentation.

**Spec:** `docs/superpowers/specs/2026-09-12-telemetry-spike-design.md` and `docs/scope/scope0.md`

## Global Constraints

- All multi-byte frame values SHALL be little-endian.
- Numeric correctness SHALL use integer or fixed-point representations with declared units; floating-point arithmetic SHALL NOT be part of the canonical contract.
- Contracts SHALL be canonical JSON documents validated with JSON Schema at control-plane/load time, not per filter invocation.
- The timed P0 path SHALL be `resident BinaryFrame -> F1 Validate -> F2 Decode -> F3 Normalize -> F4 Evaluate Rules -> F5 Classify -> F6 Route`.
- P0 workload composition SHALL be 80% valid, 5% invalid, and 15% edge/complex.
- Correctness SHALL be a hard prerequisite for performance qualification.
- This plan SHALL NOT implement a runtime, benchmark harness, broker integration, production workflow engine, UI, Kubernetes deployment, M2/M3/M4, or language-specific B0/B1 code.
- No runtime implementation may begin until the ambiguity/gap report is reviewed and all blocking decisions are resolved.

---

### Task 1: Establish the specification artifact map and ambiguity report

**Files:**
- Create: `specification/telemetry-workflow.md`
- Create: `specification/benchmark-protocol.md`
- Create: `specification/acceptance-criteria.md`
- Create: `specification/gap-ambiguity-report.md`

**Interfaces:**
- Consumes: approved design sections 2–5 and scope sections 9–29.
- Produces: a stable list of specification artifacts and a blocking/non-blocking decision register consumed by Tasks 2–6.

- [ ] **Step 1: Write the failing consistency checklist**

Create `specification/gap-ambiguity-report.md` with one row per unresolved decision and these exact columns: `ID`, `Topic`, `Current statement`, `Ambiguity`, `Impact`, `Proposed resolution`, `Status`, `Owner`. Include at least these IDs: `FRAME-001` field widths/offsets, `FRAME-002` checksum algorithm/coverage, `FRAME-003` timestamp epoch/unit, `FRAME-004` coordinate scale, `CONTRACT-001` schema version policy, `RULE-001` frozen rule table, `ERROR-001` stable error-code taxonomy, `DATA-001` fixture encoding, `BENCH-001` warm-up and sample policy, `BENCH-002` hardware/runtime metadata, and `QOS-001` provisional gate interpretation.

- [ ] **Step 2: Run the checklist against both source documents**

Run:

```text
rg -n "FRAME-001|FRAME-002|CONTRACT-001|RULE-001|ERROR-001|BENCH-001|QOS-001" specification docs
```

Expected: the command fails before the report exists and then finds every required ID after the report is written; no requirement from the approved design is silently omitted.

- [ ] **Step 3: Write the artifact index and boundaries**

In `telemetry-workflow.md`, link to the frame, data, filter, workflow, QoS, error, and rule sections. In `benchmark-protocol.md`, link to dataset, scenarios, timers, metrics, and reporting. In `acceptance-criteria.md`, link each P0/P1 gate to the metric definition that proves it.

- [ ] **Step 4: Review and mark blockers explicitly**

Set `Status` to `BLOCKING` for any value needed to generate bytes, expected output, or comparable measurements. Set `Status` to `NON-BLOCKING` only for later concerns such as P2–P4 compatibility. Do not begin implementation while any blocking row lacks a resolution.

- [ ] **Step 5: Commit the specification skeleton**

```bash
git add specification/
git commit -m "docs: map telemetry benchmark specification gaps"
```

### Task 2: Freeze the canonical telemetry frame and fixed-point semantics

**Files:**
- Modify: `specification/telemetry-workflow.md`
- Modify: `specification/gap-ambiguity-report.md`
- Create: `contracts/data/binary-frame.schema.json`
- Create: `contracts/data/validated-frame.schema.json`
- Create: `contracts/data/normalized-telemetry.schema.json`
- Create: `contracts/data/rejection.schema.json`

**Interfaces:**
- Consumes: resolved `FRAME-*` rows from Task 1.
- Produces: exact byte offsets, widths, endianness, units, ranges, fixed-point scales, checksum rule, and JSON Schema definitions for `BinaryFrame`, `ValidatedFrame`, `NormalizedTelemetry`, and `Rejection`.

- [ ] **Step 1: Define the frame table before code**

Add a table with columns `Offset`, `Width`, `Field`, `Encoding`, `Unit`, `Allowed range`, `Required`, and `Checksum coverage`. It must cover magic, version, flags, payload length, protocol, device identifier, timestamp, latitude, longitude, speed, heading, ignition, battery, bounded sensor/IO area, and checksum. State that offsets are zero-based, widths are bytes, and multi-byte integers are little-endian.

- [ ] **Step 2: Define fixed-point conversions**

For every scaled field, specify the exact integer scale and conversion equation. For example, if coordinates use scale `10^7`, document `degrees = signed_integer / 10^7` and the exact inclusive range. Use integer comparisons in all rule definitions; prohibit language-native floating-point values in the canonical representation.

- [ ] **Step 3: Define checksum and rejection behavior**

Document algorithm, initial value, polynomial or equivalent parameters, byte coverage, byte order, and whether the checksum field is excluded. Add stable rejection codes for truncation, bad magic, unsupported version, length mismatch, range violation, unsupported protocol, and checksum failure, each with required context fields.

- [ ] **Step 4: Add JSON Schemas and validate their own examples**

Use JSON Schema Draft 2020-12. Require `schema_version`, forbid undeclared properties, constrain integer ranges, and distinguish success from rejection using explicit `kind` values. Add one valid and one invalid example per schema in `specification/telemetry-workflow.md`.

- [ ] **Step 5: Update the ambiguity report and commit**

Change all resolved `FRAME-*` and `DATA-*` rows to `RESOLVED`, record the decision and rationale, then run the repository’s JSON validation command documented in the spec. Commit:

```bash
git add specification/telemetry-workflow.md specification/gap-ambiguity-report.md contracts/data/
git commit -m "docs: freeze canonical telemetry frame semantics"
```

### Task 3: Freeze F1–F6 filter contracts and deterministic rule tables

**Files:**
- Modify: `specification/telemetry-workflow.md`
- Create: `contracts/filter/f1-validate.schema.json`
- Create: `contracts/filter/f2-decode.schema.json`
- Create: `contracts/filter/f3-normalize.schema.json`
- Create: `contracts/filter/f4-evaluate-rules.schema.json`
- Create: `contracts/filter/f5-classify.schema.json`
- Create: `contracts/filter/f6-route.schema.json`
- Create: `contracts/workflow/telemetry-p0.schema.json`
- Create: `contracts/qos/p0.schema.json`
- Create: `specification/rule-table.json`

**Interfaces:**
- Consumes: data schemas and frame semantics from Task 2.
- Produces: exact input/output/error/capability contracts, ordered P0 workflow, QoS profile, and a versioned integer/fixed-point rule table.

- [ ] **Step 1: Specify each filter’s deterministic transition**

For every F1–F6, document `input`, `output`, declared errors, side effects, allocation expectations, and whether the filter may inspect execution context. F1 must stop the pipeline on malformed input; F6 must return a route enum and must not perform network or broker operations.

- [ ] **Step 2: Freeze the rule table**

Create `specification/rule-table.json` with `rule_set_id`, `version`, ordered rules, integer operands, units, comparison operator, precedence, and output labels. Include explicit behavior for equal thresholds, missing optional sensor values, and conflicting matches. No rule may rely on floating-point comparison or unspecified map iteration order.

- [ ] **Step 3: Define workflow and QoS schemas**

The workflow schema must require identity/version, typed input/output references, exactly ordered nodes F1–F6, edges connecting each node to the next, and an execution profile of `P0`. The QoS schema must encode p50/p95/p99 targets, in-process/no-network constraints, bounded memory, allocation expectations, and the provisional B1 p99 workflow-tax limit of 20%.

- [ ] **Step 4: Add contract-loading validation cases**

Document cases that must be rejected at load time: duplicate node IDs, missing schema references, invalid filter order, unknown error code, unsupported execution profile, and QoS fields contradicting P0. Document that these checks are not repeated per invocation.

- [ ] **Step 5: Resolve contract/rule blockers and commit**

Mark `CONTRACT-*`, `RULE-*`, and `QOS-*` rows resolved only after the schemas and rule table agree. Commit:

```bash
git add contracts/ specification/telemetry-workflow.md specification/rule-table.json specification/gap-ambiguity-report.md
git commit -m "docs: freeze P0 filter and workflow contracts"
```

### Task 4: Define deterministic dataset, oracle, and expected-output fixtures

**Files:**
- Create: `datasets/README.md`
- Create: `datasets/manifest.json`
- Create: `datasets/expected-results.schema.json`
- Create: `datasets/generator-spec.md`
- Create: `datasets/oracle-spec.md`
- Create: `datasets/fixtures/README.md`

**Interfaces:**
- Consumes: canonical frame, filter contracts, and rule table from Tasks 2–3.
- Produces: language-neutral generation rules, seed/manifest requirements, fixture encoding, and oracle comparison rules consumed by every future implementation.

- [ ] **Step 1: Define seed and generation algorithm**

Specify one recorded seed, deterministic PRNG requirements, generation order, and exact payload classes near 100, 250, and 500 bytes. Define the workload counts and rounding rule for 80% valid, 5% invalid, and 15% edge/complex so every implementation receives the same manifest.

- [ ] **Step 2: Define invalid and edge case coverage**

Require fixtures for truncation, bad magic/version, length mismatch, range violation, unsupported protocol, checksum failure, threshold equality, minimum/maximum legal values, optional sensor absence, and maximum bounded sensor/IO area. Each fixture must identify its expected rejection code or successful normalized output.

- [ ] **Step 3: Define oracle comparison**

Compare logical values, ordered route/classification/severity fields, and stable rejection code/context. Explicitly ignore language-specific object layout, pointer identity, field ordering where JSON semantics do not require it, and incidental metadata. Unexpected runtime/filter faults must be reported separately from typed rejections.

- [ ] **Step 4: Define fixture manifest integrity**

Require `fixture_id`, payload class, validity class, source seed, expected outcome, schema version, and SHA-256 digest. State that raw fixture bytes and expected results are immutable inputs and that generated reports must never replace them.

- [ ] **Step 5: Commit the dataset specification**

```bash
git add datasets/
git commit -m "docs: define deterministic telemetry dataset and oracle"
```

### Task 5: Freeze benchmark protocol, timing, scenarios, and reporting

**Files:**
- Modify: `specification/benchmark-protocol.md`
- Modify: `specification/acceptance-criteria.md`
- Create: `results/README.md`
- Create: `results/raw/README.md`
- Create: `results/reports/README.md`
- Create: `results/decision-matrix/README.md`

**Interfaces:**
- Consumes: dataset manifest and P0 QoS contract from Tasks 3–4.
- Produces: reproducible B0–B3 comparison protocol, raw-result schema, report fields, and acceptance-gate calculations.

- [ ] **Step 1: Define matched B0–B3 runs and timer boundaries**

State that the primary timer starts with a resident frame ready for processing and ends at the final success/rejection outcome. Exclude dataset I/O, process startup, contract loading, result writing, and orchestration from the primary timer, while reporting them separately. Define `Workflow Tax = B1 - B0`, `Observability Tax = B2 - B1`, and `Distribution Tax = B3 - B2` using matched percentile values and a documented percentage formula.

- [ ] **Step 2: Define run protocol and scenarios**

Require warmed steady-state runs, one million frames for sequential throughput, concurrency 1/2/4/8/16/32/64 where meaningful, S1 single-frame, S2 sequential, S3 concurrency, S4 approximately 10x burst/recovery, and S5 15–30 minute sustained load. Require all payload classes and workload categories in every comparable run.

- [ ] **Step 3: Define metrics and metadata**

Require p50, p95, p99, p99.9, max, frames/sec, MB/sec, CPU utilization, CPU ns/frame where available, RSS, heap, allocations/frame, allocated bytes/frame, memory growth, and applicable GC/JIT/context-switch data. Require CPU, cores, RAM, OS/kernel, runtime/compiler, governor, container limits, commit, dataset digest, and command metadata.

- [ ] **Step 4: Define raw results and report templates**

Specify machine-readable raw records with implementation, benchmark level, scenario, concurrency, payload class, percentile values, correctness count, memory metrics, metadata, and run ID. Reports must be generated from raw results and must include gate evaluation, threats to validity, and a decision matrix; hand-edited benchmark results are forbidden.

- [ ] **Step 5: Commit the protocol and reporting contract**

```bash
git add specification/benchmark-protocol.md specification/acceptance-criteria.md results/
git commit -m "docs: freeze telemetry benchmark protocol"
```

### Task 6: Perform specification review and release the implementation gate

**Files:**
- Modify: `specification/gap-ambiguity-report.md`
- Modify: `specification/telemetry-workflow.md`
- Modify: `specification/benchmark-protocol.md`
- Modify: `specification/acceptance-criteria.md`
- Create: `specification/REVIEW.md`

**Interfaces:**
- Consumes: all artifacts from Tasks 1–5.
- Produces: a signed-off review showing that every approved-design requirement maps to a frozen artifact and that no runtime work may start before all blockers are closed.

- [ ] **Step 1: Run the spec coverage review**

Create `specification/REVIEW.md` with a coverage table mapping each approved-design section—canonical representation, execution model, frame/workload, filter semantics, B0–B3, timing, scenarios, acceptance gates, decision outputs, and non-goals—to exact artifact headings and validation evidence.

- [ ] **Step 2: Run placeholder and unresolved-blocker scans**

Run:

```text
rg -n "TBD|TODO|FIXME|later|fill in|appropriate error|Similar to Task|BLOCKING" specification contracts datasets results
```

Expected: no implementation placeholder remains; `BLOCKING` may appear only in the report’s historical/status section, and the final review must list zero unresolved blocking rows.

- [ ] **Step 3: Validate cross-artifact consistency**

Check manually and record evidence that every schema reference resolves, every F1–F6 output matches the next filter input, every error code appears in the rejection schema and filter contract, every rule-table field uses declared units/scales, and every acceptance metric is defined in the benchmark protocol.

- [ ] **Step 4: Record the implementation gate**

In `REVIEW.md`, state exactly: “Runtime implementation is authorized only after this review records zero unresolved blocking decisions and the dataset/oracle fixture contract is frozen.” If any blocker remains, leave the gate closed and identify the required decision.

- [ ] **Step 5: Commit the reviewed plan artifacts**

```bash
git add specification/
git commit -m "docs: approve telemetry specification freeze"
```

## Self-Review Checklist

- [ ] Every design requirement has a task and a named artifact.
- [ ] No task assumes a runtime implementation before the ambiguity gate.
- [ ] Frame offsets, scales, checksum, error codes, rules, fixtures, and timer boundaries are all explicitly frozen.
- [ ] Later tasks consume exact schemas, files, identifiers, and formulas defined earlier.
- [ ] The plan contains no `TBD`, `TODO`, or unspecified “handle edge cases” step.
- [ ] The P0/P1 scope, non-goals, and provisional acceptance thresholds remain intact.

