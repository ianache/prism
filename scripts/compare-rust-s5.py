import argparse
import hashlib
import json
from pathlib import Path


def load(path):
    return [json.loads(line) for line in path.read_text(encoding="utf-8") .splitlines() if line.strip()]


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--baseline", type=Path, required=True)
    parser.add_argument("--remediated", type=Path, required=True)
    parser.add_argument("--output", type=Path, required=True)
    args = parser.parse_args()
    baseline, remediated = load(args.baseline), load(args.remediated)
    if not baseline or not remediated or baseline[0].get("dataset_digest") != remediated[0].get("dataset_digest"):
        raise SystemExit("dataset identity mismatch")
    common = {}
    for level in ("b0", "b1", "b2"):
        for repetition in range(1, 6):
            old = {(r.get("window_index")): r for r in baseline if r.get("level") == level and r.get("repetition") == repetition}
            new = {(r.get("window_index")): r for r in remediated if r.get("level") == level and r.get("repetition") == repetition}
            keys = sorted(set(old) & set(new))
            common[(level, repetition)] = {"common": len(keys), "baseline_only": len(set(old) - set(new)), "remediated_only": len(set(new) - set(old)), "p99_delta_percent": (new[keys[-1]]["p99_ns"] - old[keys[-1]]["p99_ns"]) / old[keys[-1]]["p99_ns"] * 100.0 if keys and old[keys[-1]]["p99_ns"] else None}
    lines = ["# S5 Stability Remediation Comparison", "", f"Baseline SHA-256: `{hashlib.sha256(args.baseline.read_bytes()).hexdigest()}`", f"Remediated SHA-256: `{hashlib.sha256(args.remediated.read_bytes()).hexdigest()}`", "", "| Level | Rep | Common windows | Baseline only | Remediated only | Last p99 delta |", "|---|---:|---:|---:|---:|---:|"]
    for (level, repetition), value in common.items():
        lines.append(f"| {level} | {repetition} | {value['common']} | {value['baseline_only']} | {value['remediated_only']} | {value['p99_delta_percent'] if value['p99_delta_percent'] is not None else 'N/D'} |")
    lines += ["", "The comparison preserves raw p99 values and does not remove outliers or change the provisional gate.", "", "This is engineering evidence and not an automatic P0 qualification."]
    args.output.write_text("\n".join(lines) + "\n", encoding="utf-8")
    print(f"wrote {args.output}")


if __name__ == "__main__":
    main()
