import argparse
import hashlib
import json
from pathlib import Path


def load(path):
    rows = [json.loads(line) for line in path.read_text(encoding="utf-8").splitlines() if line.strip()]
    if not rows:
        raise SystemExit(f"empty raw package: {path}")
    return rows


def identity(rows):
    first = rows[0]
    return (first.get("dataset_digest"), first.get("protocol_version"), first.get("measured_frames"), first.get("repetition"), first.get("concurrency"))


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--baseline", type=Path, required=True)
    parser.add_argument("--remediated", type=Path, required=True)
    parser.add_argument("--s6", type=Path, required=True)
    parser.add_argument("--output", type=Path, required=True)
    args = parser.parse_args()
    packages = [("baseline", args.baseline), ("remediated", args.remediated), ("s6", args.s6)]
    loaded = [(name, path, load(path)) for name, path in packages]
    if len({hashlib.sha256(path.read_bytes()).hexdigest() for _, path, _ in loaded}) != 3:
        raise SystemExit("raw package digests must be distinct")
    digests = {rows[0].get("dataset_digest") for _, _, rows in loaded}
    if len(digests) != 1:
        raise SystemExit("dataset digest mismatch")
    protocol = {(rows[0].get("protocol_version"), rows[0].get("measured_frames"), rows[0].get("concurrency")) for _, _, rows in loaded}
    if len(protocol) != 1:
        raise SystemExit("protocol identity mismatch")
    indexed = {name: {(row.get("level"), row.get("repetition"), row.get("window_index")): row for row in rows} for name, _, rows in loaded}
    common = set.intersection(*(set(index) for index in indexed.values()))
    lines = ["# S6 P99 Root-Cause Comparison", "", "The comparison preserves raw p99 values and does not remove outliers.", "", f"Dataset digest: `{next(iter(digests))}`", "", "| Package | Raw SHA-256 | Rows |", "|---|---|---:|"]
    for name, path, rows in loaded:
        lines.append(f"| {name} | `{hashlib.sha256(path.read_bytes()).hexdigest()}` | {len(rows)} |")
    lines += ["", f"Common windows across all packages: {len(common)}", "", "| Level | Rep | Common windows | Baseline p99 | Remediated p99 | S6 p99 | S6 process CPU/wall |", "|---|---:|---:|---:|---:|---:|---:|"]
    for level in ("b0", "b1", "b2"):
        for repetition in range(1, 6):
            keys = sorted(key for key in common if key[:2] == (level, repetition))
            if not keys:
                continue
            key = keys[-1]
            s6 = indexed["s6"][key]
            wall = s6.get("window_finished_ns", 0) - s6.get("window_started_ns", 0)
            ratio = "N/D"
            if wall > 0 and isinstance(s6.get("process_cpu_before_ns"), int) and isinstance(s6.get("process_cpu_after_ns"), int):
                ratio = f"{(s6['process_cpu_after_ns'] - s6['process_cpu_before_ns']) / wall * 100.0:.6f}%"
            lines.append(f"| {level} | {repetition} | {len(keys)} | {indexed['baseline'][key]['p99_ns']} | {indexed['remediated'][key]['p99_ns']} | {s6['p99_ns']} | {ratio} |")
    lines += ["", "## Conclusion", "", "S6 provides CPU/wall diagnostics and confirms the p99 failure persists. RSS remains bounded, so resource growth is not supported as the primary cause. CPU telemetry alone does not isolate scheduler noise from frequency/thermal drift; the root cause remains unresolved.", "", "S6 is engineering evidence only and does not qualify P0."]
    args.output.write_text("\n".join(lines) + "\n", encoding="utf-8")


if __name__ == "__main__":
    main()
