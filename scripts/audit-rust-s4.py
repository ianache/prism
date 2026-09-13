import argparse
import json
import math
from pathlib import Path

LEVELS = ("b0", "b1", "b2")
PHASES = ("baseline", "burst", "recovery")
REPETITIONS = (1, 2, 3, 4, 5)
METRICS = ("p50_ns", "p95_ns", "p99_ns", "p99_9_ns", "max_ns", "frames_per_sec", "mb_per_sec", "offered_frames_per_sec", "processed_frames_per_sec", "calibration_median_frames_per_sec", "baseline_frames_per_sec", "burst_frames_per_sec")


def fail(message):
    raise SystemExit(message)


def tax(base, compared):
    return 0.0 if base == 0 else (compared - base) / base * 100.0


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--raw", type=Path, required=True)
    parser.add_argument("--dataset", type=Path, required=True)
    args = parser.parse_args()
    try:
        rows = [json.loads(line) for line in args.raw.read_text(encoding="utf-8").splitlines() if line.strip()]
    except (OSError, json.JSONDecodeError) as error:
        fail(f"invalid JSONL: {error}")
    expected = [(level, phase, repetition) for phase in PHASES for level in LEVELS for repetition in REPETITIONS]
    actual = [(row.get("level"), row.get("phase"), row.get("repetition")) for row in rows]
    if len(rows) != 45:
        fail(f"expected 45 S4 records, got {len(rows)}")
    if actual != expected:
        fail("invalid S4 phase order or tuple set")
    if len(set(actual)) != len(actual):
        fail("duplicate S4 tuple")
    try:
        manifest = json.loads((args.dataset / "manifest.json").read_text(encoding="utf-8"))
        digest = (args.dataset / "manifest.sha256").read_text(encoding="ascii").strip()
    except (OSError, json.JSONDecodeError) as error:
        fail(f"invalid dataset metadata: {error}")

    indexed = {(row["level"], row["phase"], row["repetition"]): row for row in rows}
    for row in rows:
        if row.get("scenario") != "S4" or row.get("protocol_version") != "1.1" or row.get("concurrency") != 1:
            fail("invalid S4 protocol identity")
        if row.get("measured_frames") != 100000:
            fail("invalid measured_frames")
        if row.get("dataset_digest") != digest or row.get("dataset_id") != manifest.get("dataset_id"):
            fail("dataset identity mismatch")
        if row.get("correctness_total") != row.get("correctness_matches"):
            fail("correctness mismatch")
        if row.get("late_frames", -1) + row.get("on_time_frames", -1) != 100000:
            fail("lateness count mismatch")
        for key in METRICS + ("lateness_p50_ns", "lateness_p95_ns", "lateness_p99_ns"):
            value = row.get(key)
            if not isinstance(value, (int, float)) or isinstance(value, bool) or not math.isfinite(value) or value < 0:
                fail(f"invalid metric: {key}")
        if not math.isclose(row["baseline_frames_per_sec"] * 10, row["burst_frames_per_sec"], rel_tol=1e-9, abs_tol=1e-9):
            fail("invalid burst calibration ratio")
        if not math.isclose(row["baseline_frames_per_sec"], row["calibration_median_frames_per_sec"] * 0.8, rel_tol=1e-9, abs_tol=1e-9):
            fail("invalid baseline calibration ratio")
        if row["level"] == "b2" and set(row.get("filter_invocations") or {}) != {f"F{i}" for i in range(1, 7)}:
            fail("incomplete B2 filter evidence")
    for phase in PHASES:
        for repetition in REPETITIONS:
            b0 = indexed[("b0", phase, repetition)]
            b1 = indexed[("b1", phase, repetition)]
            b2 = indexed[("b2", phase, repetition)]
            if not math.isclose(b1["workflow_tax_percent"], tax(b0["p99_ns"], b1["p99_ns"]), rel_tol=1e-9, abs_tol=1e-9):
                fail("workflow tax mismatch")
            if not math.isclose(b2["observability_tax_percent"], tax(b1["p99_ns"], b2["p99_ns"]), rel_tol=1e-9, abs_tol=1e-9):
                fail("observability tax mismatch")
    print("ok rows=45 levels=b0,b1,b2 phases=baseline,burst,recovery frames=100000")


if __name__ == "__main__":
    main()
