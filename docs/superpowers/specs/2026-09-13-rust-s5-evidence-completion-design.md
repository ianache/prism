# Rust S5 Evidence Completion Design

**Status:** Proposed for review  
**Date:** 2026-09-13  
**Scope:** Completion and decision packaging for the published Rust S5 implementation.

## Goal

Produce the protocol-compliant external S5 evidence package and a reproducible
gate report. This increment executes the existing Rust S5 implementation over
the external 100K corpus for 900 seconds, audits every complete 100,000-frame
window, and records whether the provisional stability and memory gates pass.

## Constraints

- Use protocol v1.1, five repetitions, and exactly 100,000 measured frames in
  every comparable window.
- Use the immutable external corpus at
  `D:\02-PERSONAL\TOOLS\prism-datasets\p0-100k`.
- Write a new immutable output path; never overwrite prior raw evidence.
- Do not alter the measured Rust path, introduce B3, add a broker/transport,
  or change the 100K decision target.
- Treat correctness as a hard prerequisite for any performance gate.
- Report unavailable host metrics as `N/D` and distinguish unavailable gates
  from passed gates.
- Do not declare P0 qualification automatically; produce evidence for a later
  decision review.

## Evidence flow

1. Confirm the published commit, dataset manifest, dataset digest, toolchain,
   host metadata, and output-path availability.
2. Run the existing S5 CLI with B0/B1/B2, concurrency 1, five repetitions,
   100,000 samples per window, and 900 seconds total duration.
3. Parse every JSONL record and verify unique `(level, repetition,
   window_index)` keys, 100K frame counts, correctness, finite metrics,
   protocol identity, and dataset identity.
4. Run `scripts/audit-rust-s5.py` and retain its stdout, stderr, exit code,
   and raw output digest as audit evidence.
5. Compute first-to-last p99 change, RSS growth when both values exist, window
   counts, incomplete tails, and matched B1/B0 and B2/B1 taxes.
6. Publish a concise report containing the command, commit, digest, host
   metadata, evidence path, gate status, threats to validity, and explicit
   non-qualification status.

## Acceptance criteria

- All five repetitions exist for B0, B1, and B2.
- Every comparable record has exactly 100,000 measured frames and 100% oracle
  correctness.
- Window indices are unique and strictly increasing within each level and
  repetition; tax comparisons use the explicit intersection of available
  windows across B0/B1/B2 and report unmatched tails.
- Required latency, throughput, identity, and tax values are finite and
  correctly paired.
- RSS and p99 stability are evaluated separately; `N/D` produces an
  unavailable result rather than a pass.
- Existing Rust and Python suites remain green after the evidence run.
- No raw binary fixture or generated evidence file is committed to Git.

## Failure handling and limitations

A failed audit, missing repetition, correctness mismatch, invalid metric,
dataset mismatch, duplicate window, or output collision stops publication of
the decision report. A gate failure is recorded as a valid negative result and
does not justify changing thresholds or rerunning selectively. Thermal drift,
OS scheduling, CPU frequency changes, RSS sampling limitations, and the
single-host scope remain threats to validity. The package supports a later P0
review but is not itself a P0 qualification.
