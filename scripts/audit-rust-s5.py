import argparse
import json
import math
from pathlib import Path

LEVELS = ("b0", "b1", "b2")
REPETITIONS = (1, 2, 3, 4, 5)
METRICS = ("p50_ns", "p95_ns", "p99_ns", "p99_9_ns", "max_ns", "frames_per_sec", "mb_per_sec")


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
        manifest = json.loads((args.dataset / "manifest.json").read_text(encoding="utf-8"))
        digest = (args.dataset / "manifest.sha256").read_text(encoding="ascii").strip()
    except (OSError, json.JSONDecodeError) as error:
        fail(f"invalid input: {error}")
    keys = []
    diagnostics_enabled = any("window_started_ns" in row for row in rows)
    for row in rows:
        key = (row.get("level"), row.get("repetition"), row.get("window_index"))
        keys.append(key)
        if row.get("scenario") != "S5" or row.get("protocol_version") != "1.1" or row.get("concurrency") != 1:
            fail("invalid S5 protocol identity")
        if row.get("level") not in LEVELS or row.get("repetition") not in REPETITIONS:
            fail("invalid level or repetition")
        if not isinstance(row.get("window_index"), int) or row["window_index"] < 0:
            fail("invalid window index")
        if row.get("measured_frames") != 100000:
            fail("invalid measured_frames")
        if row.get("dataset_digest") != digest or row.get("dataset_id") != manifest.get("dataset_id"):
            fail("dataset identity mismatch")
        if row.get("correctness_total") != row.get("correctness_matches"):
            fail("correctness mismatch")
        for name in METRICS:
            value = row.get(name)
            if not isinstance(value, (int, float)) or isinstance(value, bool) or not math.isfinite(value) or value < 0:
                fail(f"invalid metric: {name}")
        if not isinstance(row.get("duration_seconds"), (int, float)) or not math.isfinite(row["duration_seconds"]) or row["duration_seconds"] <= 0:
            fail("invalid duration_seconds")
        if diagnostics_enabled:
            if not all(isinstance(row.get(name), int) and row[name] >= 0 for name in ("window_started_ns", "window_finished_ns", "repetition_elapsed_ns")):
                fail("invalid diagnostic timestamps")
            if row["window_finished_ns"] <= row["window_started_ns"]:
                fail("non-monotonic window timestamps")
            cpu_names = ("process_cpu_before_ns", "process_cpu_after_ns", "system_cpu_before_ns", "system_cpu_after_ns")
            cpu_values = [row.get(name) for name in cpu_names]
            if any(value is not None and (not isinstance(value, int) or value < 0) for value in cpu_values):
                fail("invalid diagnostic CPU metrics")
            if all(value is not None for value in cpu_values) and (row["process_cpu_after_ns"] < row["process_cpu_before_ns"] or row["system_cpu_after_ns"] < row["system_cpu_before_ns"]):
                fail("invalid diagnostic CPU ordering")
        for name in ("rss_before_bytes", "rss_after_bytes"):
            value = row.get(name)
            if value != "N/D" and (not isinstance(value, int) or value < 0):
                fail(f"invalid resource metric: {name}")
    if len(keys) != len(set(keys)):
        fail("duplicate S5 window")
    grouped = {(level, repetition): sorted((row for row in rows if row.get("level") == level and row.get("repetition") == repetition), key=lambda row: row["window_index"])
               for level in LEVELS for repetition in REPETITIONS}
    if any(not group for group in grouped.values()):
        fail("missing level or repetition window")
    for group in grouped.values():
        if [row["window_index"] for row in group] != list(range(len(group))):
            fail("non-contiguous window indices")
    for repetition in REPETITIONS:
        key_sets = [set(row["window_index"] for row in grouped[(level, repetition)]) for level in LEVELS]
        if not (key_sets[0] & key_sets[1] & key_sets[2]):
            fail("no common level window")
    indexed = {(row["level"], row["repetition"], row["window_index"]): row for row in rows}
    gate_failures = []
    for level in LEVELS:
        for repetition in REPETITIONS:
            group = grouped[(level, repetition)]
            first, last = group[0], group[-1]
            if last["p99_ns"] > first["p99_ns"] * 1.10:
                gate_failures.append(f"p99:{level}:r{repetition}")
            if first["rss_before_bytes"] != "N/D" and last["rss_after_bytes"] != "N/D" and last["rss_after_bytes"] > first["rss_before_bytes"] * 1.10:
                gate_failures.append(f"rss:{level}:r{repetition}")
    for key, b0 in list(indexed.items()):
        level, repetition, window = key
        if level == "b0":
            b1 = indexed.get(("b1", repetition, window))
            b2 = indexed.get(("b2", repetition, window))
            if b1 and not math.isclose(b1.get("workflow_tax_percent", 0.0), tax(b0["p99_ns"], b1["p99_ns"]), rel_tol=1e-9, abs_tol=1e-9):
                fail("workflow tax mismatch")
            if b1 and b2 and not math.isclose(b2.get("observability_tax_percent", 0.0), tax(b1["p99_ns"], b2["p99_ns"]), rel_tol=1e-9, abs_tol=1e-9):
                fail("observability tax mismatch")
    suffix = " gate_failures=" + ",".join(gate_failures) if gate_failures else " gates=pass"
    print(f"ok levels=b0,b1,b2 repetitions=5 frames=100000 windows={len(rows)}{suffix}")


if __name__ == "__main__":
    main()
