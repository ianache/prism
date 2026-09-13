import argparse
import hashlib
import json
import math
from pathlib import Path


def gate(name, status, detail):
    return f"| {name} | {status} | {detail} |"


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--raw", type=Path, required=True)
    parser.add_argument("--audit", type=Path, required=True)
    parser.add_argument("--output", type=Path, required=True)
    args = parser.parse_args()
    rows = [json.loads(line) for line in args.raw.read_text(encoding="utf-8").splitlines() if line.strip()]
    audit_text = args.audit.read_text(encoding="utf-8")
    raw_digest = hashlib.sha256(args.raw.read_bytes()).hexdigest()
    groups = {(level, repetition): sorted((row for row in rows if row.get("level") == level and row.get("repetition") == repetition), key=lambda row: row["window_index"])
              for level in ("b0", "b1", "b2") for repetition in range(1, 6)}
    p99_changes = []
    rss_changes = []
    for group in groups.values():
        first, last = group[0], group[-1]
        if first["p99_ns"]:
            p99_changes.append((last["p99_ns"] - first["p99_ns"]) / first["p99_ns"] * 100.0)
        if isinstance(first.get("rss_before_bytes"), int) and isinstance(last.get("rss_after_bytes"), int) and first["rss_before_bytes"]:
            rss_changes.append((last["rss_after_bytes"] - first["rss_before_bytes"]) / first["rss_before_bytes"] * 100.0)
    p99_max = max(p99_changes) if p99_changes else None
    rss_max = max(rss_changes) if rss_changes else None
    lines = ["# Rust S5 Evidence Report", "", f"Raw path: `{args.raw}`", f"Raw SHA-256: `{raw_digest}`", f"Audit output: `{args.audit}`", "", "## Audit", "", audit_text.strip(), "", "## Summary", "", f"Rows: {len(rows)}", f"Complete windows per tuple: {min(len(group) for group in groups.values())}", f"Maximum p99 change: {p99_max:.6f}%" if p99_max is not None else "Maximum p99 change: N/D", f"Maximum RSS change: {rss_max:.6f}%" if rss_max is not None else "Maximum RSS change: UNAVAILABLE", "", "## Gates", "", "| Gate | Status | Detail |", "|---|---|---|"]
    lines.append(gate("Correctness", "PASS" if all(row.get("correctness_total") == row.get("correctness_matches") for row in rows) else "FAIL", "all windows match"))
    lines.append(gate("p99 stability", "PASS" if p99_max is not None and math.isfinite(p99_max) and p99_max <= 10 else "FAIL", "threshold <= 10%" if p99_max is not None else "missing p99 series"))
    lines.append(gate("RSS growth", "PASS" if rss_max is not None and rss_max <= 10 else "UNAVAILABLE", "threshold < 10%" if rss_max is not None else "RSS not available"))
    lines += ["", "## Limitations", "", "The result is single-host engineering evidence and is not an automatic P0 qualification. Thermal drift, scheduler noise, frequency changes, and unavailable platform metrics remain threats to validity."]
    args.output.write_text("\n".join(lines) + "\n", encoding="utf-8")
    print(f"wrote {args.output} rows={len(rows)} sha256={raw_digest}")


if __name__ == "__main__":
    main()
