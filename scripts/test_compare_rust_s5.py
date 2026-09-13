import json
import subprocess
import sys
import tempfile
import unittest
from pathlib import Path

SCRIPT = Path(__file__).with_name("compare-rust-s5.py")


class CompareTests(unittest.TestCase):
    def test_comparison_reports_common_and_unmatched_windows(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            old = [{"level": "b0", "repetition": 1, "window_index": i, "p99_ns": 10, "dataset_digest": "d"} for i in range(2)]
            new = [{"level": "b0", "repetition": 1, "window_index": 0, "p99_ns": 20, "dataset_digest": "d"}]
            for level in ("b1", "b2"):
                for repetition in range(1, 6):
                    old.append({"level": level, "repetition": repetition, "window_index": 0, "p99_ns": 10, "dataset_digest": "d"})
                    new.append({"level": level, "repetition": repetition, "window_index": 0, "p99_ns": 10, "dataset_digest": "d"})
            for repetition in range(2, 6):
                old.append({"level": "b0", "repetition": repetition, "window_index": 0, "p99_ns": 10, "dataset_digest": "d"})
                new.append({"level": "b0", "repetition": repetition, "window_index": 0, "p99_ns": 10, "dataset_digest": "d"})
            a, b, out = root / "a.jsonl", root / "b.jsonl", root / "report.md"
            a.write_text("\n".join(json.dumps(r) for r in old) + "\n", encoding="utf-8")
            b.write_text("\n".join(json.dumps(r) for r in new) + "\n", encoding="utf-8")
            result = subprocess.run([sys.executable, str(SCRIPT), "--baseline", str(a), "--remediated", str(b), "--output", str(out)], capture_output=True, text=True)
            self.assertEqual(result.returncode, 0, result.stderr)
            self.assertIn("Common windows", out.read_text(encoding="utf-8"))


if __name__ == "__main__":
    unittest.main()
