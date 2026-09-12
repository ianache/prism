import unittest
from unittest.mock import patch

from tools.dataset.frame import encode_frame, mutate_frame
from tools.dataset.model import TelemetryFrame
from tools.dataset.oracle import oracle_bytes


def valid_payload() -> bytes:
    return encode_frame(TelemetryFrame(1, 0, 0, 0, 0, 0, 1, 12000, (), payload_class=105))


class OracleTests(unittest.TestCase):
    def test_valid_output(self) -> None:
        result = oracle_bytes(valid_payload())
        self.assertEqual(result["kind"], "normalized_telemetry")
        self.assertEqual(result["classification"], "NORMAL")
        self.assertEqual(result["route"], "STANDARD")

    def test_typed_rejection(self) -> None:
        result = oracle_bytes(mutate_frame(valid_payload(), "checksum_failure"))
        self.assertEqual(result["kind"], "rejection")
        self.assertEqual(result["code"], "CHECKSUM_FAILURE")

    def test_unexpected_fault_is_separate(self) -> None:
        with patch("tools.dataset.oracle.decode_frame", side_effect=RuntimeError("boom")):
            result = oracle_bytes(valid_payload())
        self.assertEqual(result["kind"], "execution_failure")
        self.assertEqual(result["error_type"], "RuntimeError")


if __name__ == "__main__":
    unittest.main()
