import json
import subprocess
import sys
import tempfile
import unittest
from pathlib import Path

SCRIPT = Path(__file__).with_name("audit-rust-s5.py")


def make_row(level, repetition, window, p99):
    return {
        "level": level, "repetition": repetition, "window_index": window, "scenario": "S5", "concurrency": 1,
        "protocol_version": "1.1", "measured_frames": 100000, "dataset_id": "d", "dataset_digest": "a" * 64,
        "correctness_total": 100000, "correctness_matches": 100000, "p50_ns": 1, "p95_ns": 2, "p99_ns": p99,
        "p99_9_ns": 3, "max_ns": 4, "frames_per_sec": 1.0, "mb_per_sec": 1.0,
        "duration_seconds": 0.2,
        "rss_before_bytes": "N/D", "rss_after_bytes": "N/D", "workflow_tax_percent": 0.0, "observability_tax_percent": 0.0,
    }


class S5AuditTests(unittest.TestCase):
    def run_audit(self, mutate=None):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory) / "dataset"
            root.mkdir()
            (root / "manifest.json").write_text(json.dumps({"dataset_id": "d"}), encoding="utf-8")
            (root / "manifest.sha256").write_text("a" * 64, encoding="ascii")
            rows = [make_row(level, repetition, window, 100) for level in ("b0", "b1", "b2") for repetition in range(1, 6) for window in range(2)]
            if mutate:
                mutate(rows)
            raw = root / "raw.jsonl"
            raw.write_text("".join(json.dumps(item) + "\n" for item in rows), encoding="utf-8")
            return subprocess.run([sys.executable, str(SCRIPT), "--dataset", str(root), "--raw", str(raw)], capture_output=True, text=True)

    def test_valid_windows(self):
        result = self.run_audit()
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertIn("frames=100000", result.stdout)

    def test_duplicate_or_wrong_frame_count_is_rejected(self):
        self.assertNotEqual(self.run_audit(lambda rows: rows[1].update(window_index=0)).returncode, 0)
        self.assertNotEqual(self.run_audit(lambda rows: rows[0].update(measured_frames=10)).returncode, 0)

    def test_different_window_counts_are_valid_when_intersection_exists(self):
        def remove_last_b2(rows):
            rows[:] = [row for row in rows if not (row["level"] == "b2" and row["window_index"] == 1)]
        self.assertEqual(self.run_audit(remove_last_b2).returncode, 0)

    def test_gate_failure_is_reported_without_rejecting_integrity(self):
        def degrade_last_b0(rows):
            for row in rows:
                if row["level"] == "b0" and row["window_index"] == 1:
                    row["p99_ns"] = 1000
                if row["level"] == "b1" and row["window_index"] == 1:
                    row["workflow_tax_percent"] = -90.0
        result = self.run_audit(degrade_last_b0)
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertIn("gate_failures", result.stdout)


if __name__ == "__main__":
    unittest.main()
