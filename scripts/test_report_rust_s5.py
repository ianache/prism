import json
import importlib.util
import sys
import tempfile
import unittest
from pathlib import Path

SPEC = importlib.util.spec_from_file_location("report_rust_s5", Path(__file__).with_name("report-rust-s5.py"))
MODULE = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(MODULE)
main = MODULE.main


class ReportTests(unittest.TestCase):
    def test_report_contains_diagnostics_and_hypotheses(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            raw = root / "raw.jsonl"
            audit = root / "audit.txt"
            output = root / "report.md"
            rows = []
            for index, p99 in enumerate((100, 120), 1):
                rows.append({"level": "b0", "repetition": 1, "window_index": index, "p99_ns": p99, "correctness_total": 1, "correctness_matches": 1, "window_started_ns": index, "window_finished_ns": index + 1, "rss_before_bytes": 100, "rss_after_bytes": 101, "process_cpu_before_ns": 10, "process_cpu_after_ns": 20, "system_cpu_before_ns": 30, "system_cpu_after_ns": 40})
            raw.write_text("\n".join(json.dumps(row) for row in rows), encoding="utf-8")
            audit.write_text("ok", encoding="utf-8")
            old = sys.argv
            try:
                sys.argv = ["report", "--raw", str(raw), "--audit", str(audit), "--output", str(output)]
                main()
            finally:
                sys.argv = old
            report = output.read_text(encoding="utf-8")
            self.assertIn("## Diagnostics", report)
            self.assertIn("## Hypotheses", report)
            self.assertIn("Window boundary ordering: PASS", report)
            self.assertIn("CPU telemetry: PRESENT", report)


if __name__ == "__main__":
    unittest.main()
