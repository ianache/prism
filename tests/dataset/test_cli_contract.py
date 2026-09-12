import subprocess
import sys
import unittest

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


if __name__ == "__main__":
    unittest.main()
