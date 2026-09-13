import json
import subprocess
import sys
import tempfile
import unittest
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
SCRIPT = ROOT / "scripts" / "audit-rust-s1.py"


def valid_row(level: str, repetition: int, digest: str) -> dict:
    return {
        "run_id": f"S1-{level}-{repetition}", "implementation": "rust",
        "level": level, "scenario": "S1", "concurrency": 1,
        "repetition": repetition, "p50_ns": 1, "p95_ns": 2,
        "p99_ns": 3, "p99_9_ns": 4, "max_ns": 5,
        "frames_per_sec": 10.0, "mb_per_sec": 1.0,
        "correctness_total": 10, "correctness_matches": 10,
        "timestamp_utc": "unix:1", "dataset_id": "dataset",
        "fixture_count": 100000, "payload_class_105": 33334,
        "payload_class_249": 33333, "payload_class_501": 33333,
        "valid_count": 80000, "invalid_count": 5000,
        "edge_complex_count": 15000, "warmup_target": 10000,
        "warmup_frames": 12000, "convergence_window": 1000,
        "convergence_threshold_percent": 5, "converged": True,
        "measured_frames": 10, "typed_rejections": 1,
        "execution_failures": 0, "command": "bench", "dataset_digest": digest,
        "workflow_tax_percent": 0.0, "metadata": {"commit": "N/D"},
    }


class AuditContractTests(unittest.TestCase):
    def run_audit(self, mutate=None):
        digest = "a" * 64
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory) / "dataset"
            root.mkdir()
            (root / "manifest.json").write_text(
                json.dumps({"dataset_id": "dataset", "workload": {"total": 100000}, "fixtures": [{}] * 100000}),
                encoding="utf-8",
            )
            (root / "manifest.sha256").write_text(digest, encoding="ascii")
            rows = [valid_row(level, repetition, digest) for level in ("b0", "b1") for repetition in range(1, 6)]
            if mutate:
                mutate(rows[0])
            raw = Path(directory) / "raw.jsonl"
            raw.write_text("".join(json.dumps(row) + "\n" for row in rows), encoding="utf-8")
            return subprocess.run(
                [sys.executable, str(SCRIPT), "--dataset", str(root), "--raw", str(raw)],
                capture_output=True, text=True, check=False,
            )

    def test_complete_record_set_is_accepted(self):
        result = self.run_audit()
        self.assertEqual(result.returncode, 0, result.stderr)

    def test_missing_protocol_field_is_rejected(self):
        result = self.run_audit(lambda row: row.pop("dataset_id"))
        self.assertNotEqual(result.returncode, 0)

    def test_non_finite_metric_is_rejected(self):
        result = self.run_audit(lambda row: row.update(frames_per_sec=float("nan")))
        self.assertNotEqual(result.returncode, 0)


if __name__ == "__main__":
    unittest.main()
