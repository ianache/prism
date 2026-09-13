import argparse
import json
import math
from pathlib import Path


LEVELS = ("b0", "b1", "b2")
CONCURRENCIES = (1, 2, 4, 8, 16, 32, 64)
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
    except (OSError, json.JSONDecodeError) as error:
        fail(f"invalid JSONL: {error}")
    expected_keys = {(level, concurrency, repetition) for level in LEVELS for concurrency in CONCURRENCIES for repetition in REPETITIONS}
    actual_keys = [(row.get("level"), row.get("concurrency"), row.get("repetition")) for row in rows]
    if len(rows) != 105:
        fail(f"expected 105 S3 records, got {len(rows)}")
    if len(set(actual_keys)) != len(actual_keys):
        fail("duplicate S3 tuple")
    if set(actual_keys) != expected_keys:
        fail("incomplete S3 level/concurrency/repetition set")

    try:
        manifest = json.loads((args.dataset / "manifest.json").read_text(encoding="utf-8"))
        digest = (args.dataset / "manifest.sha256").read_text(encoding="ascii").strip()
    except (OSError, json.JSONDecodeError) as error:
        fail(f"invalid dataset metadata: {error}")

    indexed = {(row["level"], row["concurrency"], row["repetition"]): row for row in rows}
    for row in rows:
        if row.get("scenario") != "S3" or row.get("protocol_version") != "1.1":
            fail("invalid S3 protocol identity")
        if row.get("measured_frames") != 100000:
            fail("invalid measured_frames")
        if row.get("dataset_digest") != digest or row.get("dataset_id") != manifest.get("dataset_id"):
            fail("dataset identity mismatch")
        if row.get("correctness_total") != row.get("correctness_matches"):
            fail("correctness mismatch")
        for key in METRICS:
            value = row.get(key)
            if not isinstance(value, (int, float)) or isinstance(value, bool) or not math.isfinite(value) or value < 0:
                fail(f"invalid metric: {key}")
        if row["level"] == "b2":
            evidence = row.get("filter_invocations")
            if set(evidence or {}) != {f"F{i}" for i in range(1, 7)}:
                fail("incomplete B2 filter evidence")
            if any(not isinstance(value, int) or value < 0 for value in evidence.values()):
                fail("invalid B2 filter invocation count")

    for concurrency in CONCURRENCIES:
        for repetition in REPETITIONS:
            b0 = indexed[("b0", concurrency, repetition)]
            b1 = indexed[("b1", concurrency, repetition)]
            b2 = indexed[("b2", concurrency, repetition)]
            if not math.isclose(b1["workflow_tax_percent"], tax(b0["p99_ns"], b1["p99_ns"]), rel_tol=1e-9, abs_tol=1e-9):
                fail("workflow tax mismatch")
            if not math.isclose(b2["observability_tax_percent"], tax(b1["p99_ns"], b2["p99_ns"]), rel_tol=1e-9, abs_tol=1e-9):
                fail("observability tax mismatch")

    print("ok rows=105 levels=b0,b1,b2 concurrencies=1,2,4,8,16,32,64 frames=100000")


if __name__ == "__main__":
    main()
