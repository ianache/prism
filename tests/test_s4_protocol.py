import unittest
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]


class S4ProtocolTests(unittest.TestCase):
    def test_documents_declare_s4_phases_and_cardinality(self):
        docs = (ROOT / "docs/rust-b0-b1.md").read_text(encoding="utf-8")
        self.assertIn("S4", docs)
        self.assertIn("baseline", docs)
        self.assertIn("burst", docs)
        self.assertIn("recovery", docs)
        self.assertIn("45", docs)


if __name__ == "__main__":
    unittest.main()
