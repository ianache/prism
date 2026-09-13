import unittest
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]

class S2ProtocolTests(unittest.TestCase):
    def test_documents_declare_s2_100k(self):
        protocol = (ROOT / 'specification/benchmark-protocol.md').read_text(encoding='utf-8')
        docs = (ROOT / 'docs/rust-b0-b1.md').read_text(encoding='utf-8')
        self.assertIn('S2', protocol)
        self.assertIn('100,000 measured frames per repetition', protocol)
        self.assertIn('S2', docs)

if __name__ == '__main__':
    unittest.main()
