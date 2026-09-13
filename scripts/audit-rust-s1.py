import argparse
import json
from pathlib import Path


REQUIRED = {
    "run_id", "implementation", "level", "scenario", "concurrency", "repetition",
    "p50_ns", "p95_ns", "p99_ns", "p99_9_ns", "max_ns", "frames_per_sec", "mb_per_sec",
    "correctness_total", "correctness_matches", "dataset_digest", "workflow_tax_percent", "metadata",
}


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--dataset", type=Path, required=True)
    parser.add_argument("--raw", type=Path, required=True)
    args = parser.parse_args()
    manifest = json.loads((args.dataset / "manifest.json").read_text(encoding="utf-8"))
    rows = [json.loads(line) for line in args.raw.read_text(encoding="utf-8").splitlines() if line]
    expected_digest = (args.dataset / "manifest.sha256").read_text(encoding="ascii").strip()
    if manifest["workload"]["total"] != 100000 or len(manifest["fixtures"]) != 100000:
        raise SystemExit("dataset is not the required 100000-fixture corpus")
    if len(rows) != 10 or {row["level"] for row in rows} != {"b0", "b1"}:
        raise SystemExit("expected five raw records for each level")
    for row in rows:
        missing = REQUIRED - row.keys()
        if missing:
            raise SystemExit(f"missing raw fields: {sorted(missing)}")
        if row["correctness_total"] != row["correctness_matches"]:
            raise SystemExit(f"correctness mismatch in {row['run_id']}")
        if row["dataset_digest"] != expected_digest:
            raise SystemExit(f"dataset digest mismatch in {row['run_id']}")
        if not isinstance(row["workflow_tax_percent"], (int, float)):
            raise SystemExit(f"invalid workflow tax in {row['run_id']}")
    print(f"ok rows={len(rows)} fixtures={manifest['workload']['total']} digest={expected_digest}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
