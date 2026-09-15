# Rust S21 HTTP Production Vertical Slice Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Expose the existing B0/B1/B2 telemetry core through a bounded HTTP/1.1 production flow with TLS, authentication, health/readiness, lifecycle and Docker Compose support while preserving TCP JSONL.

**Architecture:** Add an explicit HTTP protocol mode at the existing `prism-run` transport boundary. Keep route selection, workers, capacity, TLS, auth and lifecycle in the current server path; isolate HTTP parsing/response mapping from the domain dispatcher and preserve TCP as the default. Extend the existing Compose entrypoint, healthcheck and client assets instead of creating a second server implementation.

**Tech Stack:** Rust 2021, standard library transport/concurrency, rustls 0.23, Python 3 standard library for Docker helpers/contracts, Docker Compose, PowerShell/curl manual verification.

**Spec:** `docs/superpowers/specs/2026-09-14-rust-s21-http-production-vertical-slice-design.md`

## Global Constraints

- TCP JSONL remains the default and must remain compatible with S7–S20.
- HTTP scope is HTTP/1.1 request/response only; no HTTP/2, WebSockets, SSE, streaming, chunked, multipart, brokers or persistence.
- The configured route is fixed at process startup; clients cannot select B0/B1/B2 per request.
- Request bodies and protocol headers are strictly bounded; the body limit remains equivalent to 64 KiB.
- TLS continues to use the existing rustls configuration and Compose secrets.
- Production containers remain non-root as `10001:10001`.
- `100000` remains the mandatory comparable benchmark/gate target and is not reinterpreted as an HTTP request limit.
- Tokens, PEM material and sensitive paths must not appear in responses, logs or artifacts.
- Use TDD: each behavior starts with a failing focused test, then the smallest implementation, then the focused and regression suites.
- Keep `graphify-out/` untracked and out of all commits.

---

### Task 1: Add explicit protocol configuration and HTTP contract surface

**Files:**
- Modify: `crates/prism-runner/src/cli.rs`
- Modify: `crates/prism-runner/src/main.rs`
- Modify: `crates/prism-runner/src/lib.rs`
- Test: `crates/prism-runner/tests/route_contract.rs`
- Test: `crates/prism-runner/tests/protocol_contract.rs`

**Interfaces:**
- Consumes: existing `Args`, `Route`, `SecurityConfig` and TCP listener options.
- Produces: an explicit protocol value with `tcp` default and `http` alternative, validated before files/listeners are opened; shared request-processing entry points that can receive a normalized HTTP credential without changing TCP behavior.

- [ ] Write failing CLI tests for default `tcp`, accepted `http`, invalid protocol, and HTTP-only configuration combinations.
- [ ] Write failing contract tests that assert the fixed configured route remains the only route used by a request.
- [ ] Implement the smallest protocol enum/configuration change and expose it through existing argument parsing/help text.
- [ ] Refactor authentication input only as needed so TCP envelope auth and HTTP bearer auth share one internal decision without leaking credentials.
- [ ] Run focused Rust CLI/protocol tests and confirm the pre-change TCP contracts remain green.
- [ ] Commit: `feat: add HTTP protocol configuration contract`.

### Task 2: Implement a strict bounded HTTP/1.1 adapter

**Files:**
- Create: `crates/prism-runner/src/http.rs`
- Modify: `crates/prism-runner/src/lib.rs`
- Modify: `crates/prism-runner/src/output.rs` if response mapping needs a shared helper
- Test: `crates/prism-runner/tests/http_contract.rs`

**Interfaces:**
- Consumes: a stream implementing `Read`/`Write`, configured `Route`, request prefix, auth result and shared process-line/domain dispatcher.
- Produces: strict request parsing and response-writing functions with bounded headers/body; stable mappings for `200`, `400`, `401`, `404`, `405`, `413`, `415`, `500` and `503` as specified.

- [ ] Write failing parser tests for valid `POST /v1/process`, health/readiness GETs, malformed request line, unsupported method/path, missing/invalid `Content-Length`, wrong content type, oversized body and rejected chunked transfer.
- [ ] Write failing response tests for domain rejection versus transport failure, JSON content type, content length and connection-close behavior.
- [ ] Implement bounded HTTP/1.1 parsing without accepting chunked or multipart bodies; enforce explicit header and body limits.
- [ ] Implement JSON response serialization using the existing output/domain shape and ensure tokens are never copied to output.
- [ ] Run `cargo test -p prism-runner --release --test http_contract` and the existing protocol/dispatch tests.
- [ ] Commit: `feat: add bounded HTTP request adapter`.

### Task 3: Integrate HTTP with workers, TLS, auth and lifecycle

**Files:**
- Modify: `crates/prism-runner/src/transport.rs`
- Modify: `crates/prism-runner/src/lifecycle.rs` only if readiness state needs a narrow shared query
- Modify: `crates/prism-runner/src/main.rs`
- Test: `crates/prism-runner/tests/http_e2e.rs`
- Test: `crates/prism-runner/tests/s12_tls_e2e.rs` or a focused S21 TLS test module

**Interfaces:**
- Consumes: Task 1 protocol mode, Task 2 HTTP adapter, existing worker pool, capacity permits, rustls stream and shutdown sentinel.
- Produces: a real HTTP listener path using the same route/workers/queue/TLS/auth configuration, `/healthz`, `/readyz`, `DRAINING` rejection and clean stop/reconnect semantics.

- [ ] Write failing process tests that start plaintext HTTP and complete health, readiness and B0/B1/B2 processing.
- [ ] Write failing TLS tests for valid bearer auth, missing/invalid bearer auth, certificate verification and secret redaction.
- [ ] Write failing lifecycle tests for readiness before/after drain, `503` admission rejection and bounded shutdown.
- [ ] Integrate the HTTP adapter into the existing accept/worker path without changing TCP defaults or capacity semantics.
- [ ] Ensure HTTP health endpoints do not execute telemetry processing and readiness is tied to admission/drain state.
- [ ] Run focused S21 process tests plus `cargo test --workspace --release`.
- [ ] Commit: `feat: serve HTTP through the production lifecycle`.

### Task 4: Extend Docker Compose, healthcheck and client flows

**Files:**
- Modify: `docker-compose.yml`
- Modify: `docker-compose.production.yml` only where protocol-compatible production overrides are required
- Modify: `docker/entrypoint.sh`
- Modify: `docker/healthcheck.py`
- Modify: `docker/client.py`
- Modify: `.env.example`
- Test: `tests/test_s14_compose_contract.py`
- Test: `tests/test_s21_http_compose_contract.py`

**Interfaces:**
- Consumes: Task 3 protocol flag/configuration and existing TLS/non-root secret mounts.
- Produces: explicit `PRISM_PROTOCOL=tcp|http` Compose configuration, HTTP healthcheck/client behavior, safe defaults preserving TCP, and valid base/production Compose overlays.

- [ ] Write failing Python contracts for `PRISM_PROTOCOL`, HTTP entrypoint flags, `/healthz` healthcheck selection, HTTP client mode and safe `.env.example` defaults.
- [ ] Implement protocol propagation in the entrypoint while preserving TCP defaults and TERM/sentinel handling.
- [ ] Extend the healthcheck to call `/healthz`/`/readyz` in HTTP mode and retain the existing TLS JSONL check in TCP mode.
- [ ] Extend the sample client with an explicit HTTP mode using the same certificate/token files and request shape.
- [ ] Run focused Compose contracts and `docker compose config` for base and production files.
- [ ] Commit: `feat: add HTTP Compose operational flow`.

### Task 5: Add CI contracts, manual verification and evidence

**Files:**
- Modify: `.github/workflows/production-readiness.yml`
- Modify: `scripts/ci_contract.py`
- Create: `scripts/http_smoke.py` or a narrowly scoped equivalent
- Create: `docs/evidence/rust-s21-http-production-vertical-slice-2026-09-14.md`
- Modify: `docs/guia-servidor-supervisor-cliente-tls.md`
- Test: `tests/test_s21_http_contract.py`

**Interfaces:**
- Consumes: the verified HTTP server, Compose mode and existing S20 CI jobs.
- Produces: automated HTTP contract/smoke coverage, manual PowerShell and curl instructions, sanitized S21 evidence and CI visibility without weakening the existing S20 gate.

- [ ] Write failing CI-contract tests requiring HTTP endpoint/configuration coverage and sanitized smoke output.
- [ ] Add a deterministic smoke that sends three frames through HTTP and checks B2 response shape, auth and health/readiness without changing the 100K gate.
- [ ] Add a CI contract job or step after build/Compose validation; keep the existing S20 production-runtime gate intact.
- [ ] Document plaintext and TLS manual flows, including PowerShell-compatible hex generation and curl examples.
- [ ] Run the focused Python contract set, smoke syntax checks and secret scanner; inspect generated evidence for PEM/token/path leakage.
- [ ] Commit: `test: add S21 HTTP CI and manual evidence contracts`.

### Task 6: Full verification, consumption record and integration readiness

**Files:**
- Modify: `docs/consumo.md`
- Modify: `docs/evidence/rust-s21-http-production-vertical-slice-2026-09-14.md`

**Interfaces:**
- Consumes: all S21 implementation, test, Compose and CI artifacts.
- Produces: final auditable evidence and a branch ready for review/merge.

- [ ] Run `python -m pytest -q` and record the exact result.
- [ ] Run `cargo test --workspace --release` and record the exact result.
- [ ] Run base/production `docker compose config`, HTTP focused process tests, `git diff --check` and the secret scan.
- [ ] Confirm the comparable target remains exactly `100000` wherever benchmark/gate criteria are asserted.
- [ ] Update `docs/consumo.md` with one row per S21 task and one final plan row, using real elapsed time/tokens when available and `N/D` otherwise.
- [ ] Complete the S21 evidence with commit, test results, smoke results and known limitations; do not claim the local Linux runner is implemented.
- [ ] Commit: `docs: finalize S21 HTTP vertical slice evidence`.

## Verification Matrix

| Area | Required evidence |
|---|---|
| Compatibility | TCP JSONL default and existing S7–S20 tests remain green |
| HTTP contract | Parser, status mapping, limits, auth and response-shape tests |
| Runtime | Plaintext and TLS process tests for health/readiness and B0/B1/B2 |
| Lifecycle | Drain, `503`, stop/restart and reconnect behavior |
| Compose | Base and production config for `tcp` and `http` |
| Security | No tokens/PEM in logs, responses or artifacts; non-root `10001:10001` preserved |
| Benchmark governance | `100000` remains unchanged for comparable benchmark/gate decisions |
| Documentation | curl + PowerShell manual flows and sanitized S21 evidence |

## Completion Gate

S21 is complete only when all focused and full suites pass, the HTTP flow is
verified in plaintext and TLS, Compose config is valid in both modes, artifacts
are sanitized, `docs/consumo.md` is updated, and the branch is ready for the
standard review/merge decision. The local Linux runner remains a separate
backlog item.
