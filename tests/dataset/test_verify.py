import json
import tempfile
import unittest
from pathlib import Path

from tools.dataset.generate import SEED, generate_dataset
from tools.dataset.verify import verify_dataset


class VerifyTests(unittest.TestCase):
    def setUp(self) -> None:
        self.temp = tempfile.TemporaryDirectory()
        self.dataset = Path(self.temp.name) / "dataset"
        generate_dataset(self.dataset, SEED, 30)

    def tearDown(self) -> None:
        self.temp.cleanup()

    def test_valid_dataset(self) -> None:
        verify_dataset(self.dataset)

    def test_changed_bytes_fail(self) -> None:
        path = next((self.dataset / "fixtures").glob("*.bin"))
        original = path.read_bytes()
        path.write_bytes(bytes([original[0] ^ 1]) + original[1:])
        with self.assertRaisesRegex(ValueError, "digest"):
            verify_dataset(self.dataset)

    def test_missing_fixture_fails(self) -> None:
        next((self.dataset / "fixtures").glob("*.bin")).unlink()
        with self.assertRaisesRegex(ValueError, "missing fixture"):
            verify_dataset(self.dataset)

    def test_wrong_expected_result_fails(self) -> None:
        path = self.dataset / "expected-results.jsonl"
        records = [json.loads(line) for line in path.read_text(encoding="utf-8").splitlines()]
        records[0]["expected"] = {"kind": "rejection", "code": "BAD_MAGIC"}
        path.write_text(chr(10).join(json.dumps(record, sort_keys=True, separators=(",", ":")) for record in records) + chr(10), encoding="utf-8")
        with self.assertRaisesRegex(ValueError, "oracle mismatch"):
            verify_dataset(self.dataset)


if __name__ == "__main__":
    unittest.main()
