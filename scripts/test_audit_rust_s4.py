import json
import subprocess
import sys
import tempfile
import unittest
from pathlib import Path

SCRIPT = Path(__file__).with_name("audit-rust-s4.py")
PHASES = ("baseline", "burst", "recovery")


def row(level, phase, repetition, p99):
    return {
        "level": level, "phase": phase, "repetition": repetition, "scenario": "S4", "concurrency": 1,
        "protocol_version": "1.1", "measured_frames": 100000, "dataset_id": "d", "dataset_digest": "a" * 64,
        "correctness_total": 100000, "correctness_matches": 100000, "p50_ns": 1, "p95_ns": 2,
        "p99_ns": p99, "p99_9_ns": 3, "max_ns": 4, "frames_per_sec": 1.0, "mb_per_sec": 1.0,
        "offered_frames_per_sec": 1.0, "processed_frames_per_sec": 1.0, "late_frames": 0,
        "on_time_frames": 100000, "lateness_p50_ns": 0, "lateness_p95_ns": 0, "lateness_p99_ns": 0,
        "calibration_median_frames_per_sec": 100.0, "baseline_frames_per_sec": 80.0, "burst_frames_per_sec": 800.0,
        "workflow_tax_percent": 10.0 if level == "b1" else 0.0,
        "observability_tax_percent": 9.090909090909092 if level == "b2" else 0.0,
        "filter_invocations": {f"F{i}": 1 for i in range(1, 7)},
    }


class S4AuditTests(unittest.TestCase):
    def run_audit(self, mutate=None):
        with tempfile.TemporaryDirectory(dir=r"D:\02-PERSONAL\TOOLS\prism-datasets") as directory:
            root = Path(directory) / "dataset"
            root.mkdir()
            (root / "manifest.json").write_text(json.dumps({"dataset_id": "d"}), encoding="utf-8")
            (root / "manifest.sha256").write_text("a" * 64, encoding="ascii")
            rows = [row(level, phase, repetition, 100 if level == "b0" else 110 if level == "b1" else 120)
                    for phase in PHASES for level in ("b0", "b1", "b2") for repetition in range(1, 6)]
            if mutate:
                mutate(rows)
            raw = root / "raw.jsonl"
            raw.write_text("".join(json.dumps(item) + "\n" for item in rows), encoding="utf-8")
            return subprocess.run([sys.executable, str(SCRIPT), "--dataset", str(root), "--raw", str(raw)], capture_output=True, text=True)

    def test_valid_package(self):
        result = self.run_audit()
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertIn("rows=45", result.stdout)

    def test_missing_phase_is_rejected(self):
        result = self.run_audit(lambda rows: rows.pop())
        self.assertNotEqual(result.returncode, 0)

    def test_wrong_protocol_metric_and_tax_are_rejected(self):
        mutations = [
            lambda rows: rows[0].__setitem__("phase", "invalid"),
            lambda rows: rows[0].__setitem__("measured_frames", 10),
            lambda rows: rows[0].__setitem__("frames_per_sec", float("nan")),
            lambda rows: rows[15].__setitem__("correctness_matches", 0),
            lambda rows: rows[5].__setitem__("workflow_tax_percent", 0.0),
            lambda rows: rows[10].__setitem__("filter_invocations", {"F1": 1}),
        ]
        for mutate in mutations:
            with self.subTest(mutate=mutate):
                self.assertNotEqual(self.run_audit(mutate).returncode, 0)


if __name__ == "__main__":
    unittest.main()
