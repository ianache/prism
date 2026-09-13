import unittest
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]


class S5ProtocolTests(unittest.TestCase):
    def test_protocol_freezes_100k_windows_and_five_repetitions(self):
        text = (ROOT / "specification" / "benchmark-protocol.md").read_text(encoding="utf-8")
        self.assertIn("100,000 measured frames per repetition", text)
        self.assertIn("S5 runs 15-30 minutes", text)

    def test_s5_design_freezes_window_contract(self):
        text = (ROOT / "docs" / "superpowers" / "specs" / "2026-09-13-rust-s5-sustained-load-design.md").read_text(encoding="utf-8")
        for marker in ("scenario S5", "window_index", "duration_seconds", "900-second", "RSS"):
            self.assertIn(marker, text)


if __name__ == "__main__":
    unittest.main()
