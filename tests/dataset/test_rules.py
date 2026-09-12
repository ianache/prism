import unittest

from tools.dataset.model import SensorRecord, TelemetryFrame
from tools.dataset.rules import evaluate_rules, load_rule_table


def make_frame(**overrides) -> TelemetryFrame:
    values = dict(device_id=1, timestamp_unix_s=0, latitude_e7=0, longitude_e7=0, speed_cm_per_s=0, heading_cdeg=0, ignition=1, battery_mv=12000, sensors=(), payload_class=105)
    values.update(overrides)
    return TelemetryFrame(**values)


class RuleTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        cls.table = load_rule_table()

    def test_equality_and_thresholds(self) -> None:
        self.assertEqual(evaluate_rules(make_frame(ignition=0), self.table)["label"], "PARKED")
        self.assertEqual(evaluate_rules(make_frame(battery_mv=11000), self.table)["label"], "NORMAL")
        self.assertEqual(evaluate_rules(make_frame(speed_cm_per_s=2778), self.table)["label"], "MOVING")
        self.assertEqual(evaluate_rules(make_frame(sensors=(SensorRecord(1, 1, 85000),)), self.table)["label"], "OVERHEAT")

    def test_missing_sensor_does_not_match(self) -> None:
        self.assertNotEqual(evaluate_rules(make_frame(), self.table)["label"], "OVERHEAT")

    def test_first_match_wins(self) -> None:
        self.assertEqual(evaluate_rules(make_frame(ignition=0, speed_cm_per_s=2778), self.table)["rule_id"], "R001")


if __name__ == "__main__":
    unittest.main()
