import subprocess
import sys
import tempfile
import unittest
from pathlib import Path

from tools.dataset.oracle import oracle_bytes
from tools.dataset.model import SensorRecord, TelemetryFrame


class CliContractTests(unittest.TestCase):
    def test_public_dataclasses_import(self) -> None:
        self.assertEqual(SensorRecord(1, 1, 2).id, 1)
        self.assertEqual(TelemetryFrame(1, 0, 0, 0, 0, 0, 0, 0, ()).protocol, 1)

    def test_empty_payload_is_typed_rejection(self) -> None:
        result = oracle_bytes(b"")
        self.assertEqual(result["kind"], "rejection")
        self.assertEqual(result["code"], "TRUNCATED")

    def test_commands_accept_help(self) -> None:
        for module in ("tools.dataset.generate", "tools.dataset.oracle", "tools.dataset.verify"):
            result = subprocess.run(
                [sys.executable, "-m", module, "--help"],
                check=False,
                capture_output=True,
                text=True,
            )
            self.assertEqual(result.returncode, 0, result.stderr)
            self.assertIn("usage:", result.stdout.lower())

    def test_invalid_seed_is_rejected(self) -> None:
        with tempfile.TemporaryDirectory() as root:
            result = subprocess.run(
                [sys.executable, "-m", "tools.dataset.generate", "--output", str(Path(root) / "dataset"), "--seed", "bad", "--count", "1"],
                check=False, capture_output=True, text=True,
            )
            self.assertNotEqual(result.returncode, 0)

    def test_nonempty_output_requires_replace(self) -> None:
        with tempfile.TemporaryDirectory() as root:
            output = Path(root) / "dataset"
            command = [sys.executable, "-m", "tools.dataset.generate", "--output", str(output), "--seed", "0x505249534D5F5631", "--count", "3"]
            self.assertEqual(subprocess.run(command, check=False).returncode, 0)
            self.assertNotEqual(subprocess.run(command, check=False, capture_output=True).returncode, 0)

    def test_verify_missing_input_is_rejected(self) -> None:
        result = subprocess.run(
            [sys.executable, "-m", "tools.dataset.verify", "--dataset", "does-not-exist"],
            check=False, capture_output=True, text=True,
        )
        self.assertNotEqual(result.returncode, 0)

    def test_oracle_refuses_existing_output(self) -> None:
        with tempfile.TemporaryDirectory() as root:
            input_dir = Path(root) / "fixtures"
            input_dir.mkdir()
            (input_dir / "sample.bin").write_bytes(b"")
            output = Path(root) / "expected-results.jsonl"
            output.write_text("sentinel\n", encoding="utf-8")
            result = subprocess.run(
                [sys.executable, "-m", "tools.dataset.oracle", "--input", str(input_dir), "--output", str(output)],
                check=False, capture_output=True, text=True,
            )
            self.assertNotEqual(result.returncode, 0)
            self.assertEqual(output.read_text(encoding="utf-8"), "sentinel\n")


if __name__ == "__main__":
    unittest.main()
