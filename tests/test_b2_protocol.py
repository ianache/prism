import unittest
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]


class B2ProtocolTests(unittest.TestCase):
    def test_protocol_declares_versioned_100k_decision_baseline(self):
        protocol = (ROOT / "specification/benchmark-protocol.md").read_text(encoding="utf-8")
        acceptance = (ROOT / "specification/acceptance-criteria.md").read_text(encoding="utf-8")
        docs = (ROOT / "docs/rust-b0-b1.md").read_text(encoding="utf-8")

        self.assertIn("1.1", protocol)
        self.assertIn("100,000 measured frames per repetition", protocol)
        self.assertIn("100,000", acceptance)
        self.assertNotIn("1,000,000", protocol)
        self.assertIn("historical", docs.lower())


if __name__ == "__main__":
    unittest.main()
