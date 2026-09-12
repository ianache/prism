import unittest

from tools.dataset.frame import decode_frame, encode_frame, mutate_frame
from tools.dataset.model import SensorRecord, TelemetryFrame


def frame(payload_class: int, sensor_count: int = 0) -> TelemetryFrame:
    sensors = tuple(SensorRecord(index + 1, 1, 20000 + index) for index in range(sensor_count))
    return TelemetryFrame(123456789012345, 1700000000, -120000000, -770000000, 1250, 9000, 1, 12400, sensors, payload_class=payload_class)


class FrameTests(unittest.TestCase):
    def test_round_trip_all_payload_classes_and_zero_sensors(self) -> None:
        for payload_class, sensor_count in ((105, 0), (249, 34), (501, 76)):
            payload = encode_frame(frame(payload_class, sensor_count))
            self.assertEqual(len(payload), payload_class)
            decoded, rejection = decode_frame(payload)
            self.assertIsNone(rejection)
            self.assertEqual(decoded, frame(payload_class, sensor_count))

    def test_mutations_have_stable_codes(self) -> None:
        payload = encode_frame(frame(105, 0))
        expected = {
            "truncation": "TRUNCATED",
            "bad_magic": "BAD_MAGIC",
            "bad_version": "UNSUPPORTED_VERSION",
            "length_mismatch": "LENGTH_MISMATCH",
            "range_violation": "RANGE_VIOLATION",
            "unsupported_protocol": "UNSUPPORTED_PROTOCOL",
            "checksum_failure": "CHECKSUM_FAILURE",
        }
        for mutation, code in expected.items():
            _, rejection = decode_frame(mutate_frame(payload, mutation))
            self.assertIsNotNone(rejection)
            self.assertEqual(rejection.code, code)

    def test_precedence_is_truncation_then_magic(self) -> None:
        payload = bytearray(encode_frame(frame(105)))
        payload[0] = 0
        _, rejection = decode_frame(bytes(payload[:-1]))
        self.assertEqual(rejection.code, "TRUNCATED")

    def test_inclusive_boundaries(self) -> None:
        base = frame(105)
        for value in (0, 4102444800):
            candidate = TelemetryFrame(base.device_id, value, base.latitude_e7, base.longitude_e7, base.speed_cm_per_s, base.heading_cdeg, base.ignition, base.battery_mv, (), payload_class=105)
            decoded, rejection = decode_frame(encode_frame(candidate))
            self.assertIsNone(rejection)
            self.assertEqual(decoded.timestamp_unix_s, value)


if __name__ == "__main__":
    unittest.main()
