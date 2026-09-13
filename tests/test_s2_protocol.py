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

    def test_documents_declare_s3_concurrency_100k(self):
        docs = (ROOT / 'docs/rust-b0-b1.md').read_text(encoding='utf-8')
        self.assertIn('S3', docs)
        self.assertIn('1, 2, 4, 8, 16, 32, 64', docs)
        self.assertIn('105', docs)

if __name__ == '__main__':
    unittest.main()
