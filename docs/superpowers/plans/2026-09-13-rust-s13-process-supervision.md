# Rust S13 Process Supervision and Release Hardening Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Provide reproducible start, readiness, health, and controlled-stop commands for the secure local runner.

**Architecture:** A PowerShell supervisor owns a bounded runtime directory and sentinel lifecycle while the existing `prism-run` binary remains the data-plane process. Readiness and health use the real TLS/authenticated JSONL path.

**Tech Stack:** PowerShell, Rust `prism-run`, Rustls 0.23, existing Cargo/Python suites.

**Spec:** `docs/superpowers/specs/2026-09-13-rust-s13-process-supervision-design.md`

## Global Constraints

- Preserve the 100,000-frame target and S7–S12 contracts.
- No secret contents in output or state files.
- No service registration, broker, persistence, remote deployment, mTLS, rotation, or P0 claim.

### Task 1: Define supervisor state and safe process identity

**Files:** `scripts/s13_supervisor.ps1`, `docs/consumo.md`

- [ ] Define explicit `start`, `check`, and `stop` actions.
- [ ] Store PID, address, sentinel, and stderr paths only under a generated runtime directory.
- [ ] Reject missing/stale state without killing unrelated processes.

### Task 2: Implement secure start and readiness

**Files:** `scripts/s13_supervisor.ps1`, `docs/rust-b0-b1.md`

- [ ] Launch release `prism-run` with TLS and token paths supplied by the operator.
- [ ] Wait for `READY` with a bounded timeout and persist non-secret state.
- [ ] Return a stable machine-readable success line.

### Task 3: Implement TLS/auth health check

**Files:** `scripts/s13_supervisor.ps1`

- [ ] Connect with `SslStream` using an explicit test/operator certificate callback.
- [ ] Send an authenticated JSONL request and validate `ok` plus route.
- [ ] Never echo the token or certificate material.

### Task 4: Implement controlled stop and recovery checks

**Files:** `scripts/s13_supervisor.ps1`, `tests/test_s13_supervision.py`

- [ ] Create the sentinel and verify `DRAINING` then `STOPPED`.
- [ ] Detect stale state and preserve unrelated processes.
- [ ] Test start/check/stop as an operator flow with a temporary fixture setup.

### Task 5: Final verification and publication

**Files:** `docs/evidence/rust-s13-process-supervision-2026-09-13.md`, `docs/consumo.md`

- [ ] Run Cargo release, Python tests, format, and diff checks.
- [ ] Record operator-flow results and explicit non-goals.
- [ ] Commit and push only verified changes.
