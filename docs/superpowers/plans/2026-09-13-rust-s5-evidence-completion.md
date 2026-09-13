# Rust S5 Evidence Completion Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Produce and audit the protocol-compliant Rust S5 external evidence package without changing the measured runtime.

**Architecture:** Use the published S5 CLI and auditor as fixed inputs. A preflight script captures repository, dataset, toolchain, host, and output-path identity; the benchmark emits immutable JSONL; a reporting script derives p99/RSS gates and a Markdown decision report from the raw records.

**Tech Stack:** Rust Cargo release binary, Python 3 standard library, PowerShell, canonical JSONL, SHA-256.

**Spec:** `docs/superpowers/specs/2026-09-13-rust-s5-evidence-completion-design.md`

## Global Constraints

- Use protocol v1.1, five repetitions, and exactly 100,000 measured frames per complete window.
- Use `D:\02-PERSONAL\TOOLS\prism-datasets\p0-100k` and a new immutable raw output path.
- Use the published S5 implementation; do not alter the measured Rust path or the 100K target.
- Treat correctness as a prerequisite; distinguish failed, passed, and unavailable gates.
- Do not commit raw JSONL, binary fixtures, host-specific temporary logs, or claim P0 qualification.

---

### Task 1: Freeze preflight identity and evidence paths

**Files:**
- Create: `scripts/prepare-rust-s5-evidence.ps1`
- Create: `scripts/test_prepare_rust_s5_evidence.py`
- Modify: `docs/rust-b0-b1.md`

**Interfaces:**
- Consumes: repository root, published `main` commit, external dataset path, and requested output path.
- Produces: a preflight JSON/Markdown manifest with commit, dataset ID/digest, toolchain, host metadata, command arguments, and output-path availability; exits nonzero on collisions or missing identity.

- [ ] **Step 1: Write failing preflight tests.** Test that the helper rejects an existing raw output, rejects a missing `manifest.json` or digest, and records the exact 100K/5/900 S5 parameters.
- [ ] **Step 2: Run `python -m unittest scripts.test_prepare_rust_s5_evidence` and verify failure.**
- [ ] **Step 3: Implement the PowerShell preflight.** Resolve the repository and dataset paths, read the manifest and SHA-256 file, query Git commit/toolchain/host values, and write only a small evidence manifest outside Git.
- [ ] **Step 4: Add the documented invocation and run the focused tests.**
- [ ] **Step 5: Commit with `chore: prepare S5 evidence identity`.**

### Task 2: Harden the S5 audit and gate calculations

**Files:**
- Modify: `scripts/audit-rust-s5.py`
- Modify: `scripts/test_audit_rust_s5.py`
- Create: `scripts/report-rust-s5.py`

**Interfaces:**
- Consumes: S5 JSONL plus dataset manifest/digest.
- Produces: strict audit exit status and `report-rust-s5.py --raw PATH --audit PATH --output PATH` Markdown with window counts, first/last p99 change, RSS growth, tax ranges, and gate status.

- [ ] **Step 1: Add failing tests for gate distinctions.** Cover numeric RSS pass/fail, `N/D` unavailable RSS, p99 regression above 10%, absent paired B0/B1/B2 windows, and a report that preserves negative taxes.
- [ ] **Step 2: Run `python -m unittest scripts.test_audit_rust_s5` and verify the new cases fail.**
- [ ] **Step 3: Require complete level tuples and explicit intersections.** Validate five repetitions for each level, unique contiguous window indices, 100K frames, finite metrics, correctness, identity, and B1/B0 plus B2/B1 tax pairing for every common window; report unmatched level tails separately.
- [ ] **Step 4: Implement the Markdown report.** Read only audited JSONL, classify each gate as PASS, FAIL, or UNAVAILABLE, include raw/audit SHA-256 digests, and never convert unavailable metrics into passes.
- [ ] **Step 5: Run `python -m unittest scripts.test_audit_rust_s5 scripts.test_audit_rust_s4` and commit with `test: complete S5 gate audit`.**

### Task 3: Execute the controlled external S5 run

**Files:**
- Modify: `docs/superpowers/plans/2026-09-13-rust-s5-evidence-completion.md`
- External only: `D:\02-PERSONAL\TOOLS\prism-datasets\p0-100k-s5.jsonl`
- External only: `D:\02-PERSONAL\TOOLS\prism-datasets\p0-100k-s5-preflight.json`

**Interfaces:**
- Consumes: published S5 binary, preflight manifest, and external 100K dataset.
- Produces: one immutable 900-second raw package and retained command/audit logs.

- [ ] **Step 1: Run fresh preflight.** Confirm the output path does not exist, the dataset digest is unchanged, the repository is clean, and the binary is built in release mode.
- [ ] **Step 2: Run the exact benchmark.**

```powershell
cargo run -p prism-bench --release -- `
  --dataset D:\02-PERSONAL\TOOLS\prism-datasets\p0-100k `
  --levels b0,b1,b2 --scenario S5 --concurrency 1 `
  --warmup 10000 --samples 100000 --repetitions 5 --duration-seconds 900 `
  --output D:\02-PERSONAL\TOOLS\prism-datasets\p0-100k-s5.jsonl `
  --cpu-model recorded-by-preflight --cores recorded-by-preflight `
  --ram-bytes recorded-by-preflight --os recorded-by-preflight `
  --governor recorded-by-preflight --affinity recorded-by-preflight
```

- [ ] **Step 3: Preserve the raw output and process exit code.** Do not rerun selectively after a gate failure; use a new versioned path for any repeat.
- [ ] **Step 4: Commit only the plan/status update, never the external raw package.**

### Task 4: Audit, report, and review the evidence

**Files:**
- Create: `docs/evidence/rust-s5-2026-09-13.md`
- Modify: `docs/consumo.md`
- Modify: `docs/superpowers/plans/2026-09-13-rust-s5-evidence-completion.md`

**Interfaces:**
- Consumes: external raw JSONL, preflight manifest, audit output, and generated gate report.
- Produces: a reproducible evidence note with paths, digests, gate classifications, limitations, and no P0 conclusion.

- [ ] **Step 1: Parse every raw line with `json.loads`.** Record total rows, rows per level/repetition, window index ranges, incomplete tails, and any duplicate or missing key.
- [ ] **Step 2: Run the strict audit.**

```powershell
python scripts/audit-rust-s5.py `
  --dataset D:\02-PERSONAL\TOOLS\prism-datasets\p0-100k `
  --raw D:\02-PERSONAL\TOOLS\prism-datasets\p0-100k-s5.jsonl
```

- [ ] **Step 3: Generate the Markdown gate report and compare its digests with the retained files.**
- [ ] **Step 4: Document threats to validity.** Include thermal/frequency drift, scheduler noise, RSS availability, single-host scope, and the fact that a gate result is not P0 qualification.
- [ ] **Step 5: Add one consumption row for each completed task and commit with `docs: record S5 evidence results`.**

### Task 5: Final verification and publication

**Files:**
- Modify: `docs/rust-b0-b1.md`
- Modify: `docs/evidence/rust-s5-2026-09-13.md`
- Modify: `docs/consumo.md`

- [ ] **Step 1: Run fresh verification on the published tree.**

```powershell
cargo test --workspace --release
python -m unittest discover -s scripts -p test_*.py
python -m unittest tests.test_s5_protocol
git diff --check
git ls-files "*100k*"
```

- [ ] **Step 2: Verify no binary fixture or raw external JSONL is tracked.**
- [ ] **Step 3: Mark the plan and each task with actual status, time, and provider measurements; use `N/D` where unavailable.**
- [ ] **Step 4: Merge the isolated documentation changes into `main` only after the evidence report is internally consistent, then publish to `origin/main`.**
- [ ] **Step 5: Report the commit, external evidence paths, audit result, gate statuses, and any pending/unavailable criteria without claiming P0 qualification.**

## Self-review

- Spec coverage: Task 1 covers identity/path safety; Task 2 covers strict auditing and gate classification; Task 3 covers the exact external run; Task 4 covers evidence reporting; Task 5 covers final verification and publication.
- No implementation placeholders remain; every command, path, input, output, and failure condition is specified.
- The 100,000-frame decision target is preserved throughout; the 900-second duration is the sustained-load budget, not a replacement for the frame target.
