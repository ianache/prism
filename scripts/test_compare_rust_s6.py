import json
import subprocess
import sys
import tempfile
import unittest
from pathlib import Path


SCRIPT = Path(__file__).with_name("compare-rust-s6.py")


class CompareS6Tests(unittest.TestCase):
    def test_comparison_requires_distinct_packages_and_reports_cpu(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            rows = []
            for level in ("b0", "b1", "b2"):
                for repetition in range(1, 6):
                    rows.append({"level": level, "repetition": repetition, "window_index": 0, "dataset_digest": "d", "protocol_version": "1.1", "measured_frames": 100000, "concurrency": 1, "p99_ns": 10, "window_started_ns": 1, "window_finished_ns": 101, "process_cpu_before_ns": 10, "process_cpu_after_ns": 60})
            paths = [root / name for name in ("baseline.jsonl", "remediated.jsonl", "s6.jsonl")]
            for index, path in enumerate(paths):
                rows[0]["p99_ns"] = 10 + index
                path.write_text("\n".join(json.dumps(row) for row in rows) + "\n", encoding="utf-8")
            output = root / "comparison.md"
            result = subprocess.run([sys.executable, str(SCRIPT), "--baseline", str(paths[0]), "--remediated", str(paths[1]), "--s6", str(paths[2]), "--output", str(output)], capture_output=True, text=True)
            self.assertEqual(result.returncode, 0, result.stderr)
            self.assertIn("process CPU/wall", output.read_text(encoding="utf-8"))


if __name__ == "__main__":
    unittest.main()
