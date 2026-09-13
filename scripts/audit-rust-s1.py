import argparse
import json
import math
from pathlib import Path


REQUIRED = {
    "run_id", "implementation", "level", "scenario", "concurrency", "repetition",
    "p50_ns", "p95_ns", "p99_ns", "p99_9_ns", "max_ns", "frames_per_sec", "mb_per_sec",
    "correctness_total", "correctness_matches", "timestamp_utc", "dataset_id", "fixture_count",
    "payload_class_105", "payload_class_249", "payload_class_501", "valid_count", "invalid_count",
    "edge_complex_count", "warmup_target", "warmup_frames", "convergence_window",
    "convergence_threshold_percent", "converged", "measured_frames", "typed_rejections",
    "execution_failures", "command", "dataset_digest", "workflow_tax_percent", "metadata",
}

INTEGER_FIELDS = {
    "concurrency", "repetition", "p50_ns", "p95_ns", "p99_ns", "p99_9_ns", "max_ns",
    "correctness_total", "correctness_matches", "fixture_count", "payload_class_105",
    "payload_class_249", "payload_class_501", "valid_count", "invalid_count", "edge_complex_count",
    "warmup_target", "warmup_frames", "convergence_window", "convergence_threshold_percent",
    "measured_frames", "typed_rejections", "execution_failures",
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
    if len(rows) != 10 or {row.get("level") for row in rows} != {"b0", "b1"}:
        raise SystemExit("expected five raw records for each level")
    by_level = {level: sorted(row["repetition"] for row in rows if row.get("level") == level) for level in ("b0", "b1")}
    if by_level != {"b0": [1, 2, 3, 4, 5], "b1": [1, 2, 3, 4, 5]}:
        raise SystemExit("expected repetitions 1..5 for each level")
    baseline = {row["repetition"]: row["p99_ns"] for row in rows if row["level"] == "b0"}
    for row in rows:
        missing = REQUIRED - row.keys()
        if missing:
            raise SystemExit(f"missing raw fields: {sorted(missing)}")
        if row["correctness_total"] != row["correctness_matches"]:
            raise SystemExit(f"correctness mismatch in {row['run_id']}")
        if row["dataset_id"] != manifest["dataset_id"] or row["fixture_count"] != 100000:
            raise SystemExit(f"dataset identity mismatch in {row['run_id']}")
        if not row["converged"] or row["warmup_target"] != 10000 or row["convergence_window"] != 1000:
            raise SystemExit(f"invalid convergence evidence in {row['run_id']}")
        if row["warmup_frames"] < row["warmup_target"] or row["measured_frames"] <= 0:
            raise SystemExit(f"invalid frame counts in {row['run_id']}")
        if any(not isinstance(row[field], int) or row[field] < 0 for field in INTEGER_FIELDS):
            raise SystemExit(f"invalid integer metric in {row['run_id']}")
        if any(not isinstance(row[field], (int, float)) or not math.isfinite(row[field]) for field in ("frames_per_sec", "mb_per_sec", "workflow_tax_percent")):
            raise SystemExit(f"invalid floating metric in {row['run_id']}")
        if not isinstance(row["metadata"], dict) or not row["command"] or not row["timestamp_utc"]:
            raise SystemExit(f"incomplete invocation metadata in {row['run_id']}")
        if row["dataset_digest"] != expected_digest:
            raise SystemExit(f"dataset digest mismatch in {row['run_id']}")
        expected_tax = 0.0 if row["level"] == "b0" else (row["p99_ns"] - baseline[row["repetition"]]) / baseline[row["repetition"]] * 100.0
        if not math.isclose(row["workflow_tax_percent"], expected_tax, rel_tol=1e-9, abs_tol=1e-9):
            raise SystemExit(f"workflow tax mismatch in {row['run_id']}")
    print(f"ok rows={len(rows)} fixtures={manifest['workload']['total']} digest={expected_digest}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
