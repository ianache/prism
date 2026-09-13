import json
import subprocess
import unittest
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
RUNNER = ROOT / "target" / "release" / "prism-run.exe"
FIXTURE = next((ROOT / "tests" / "fixtures" / "p0-smoke" / "fixtures").glob("*.bin"))


class S7VerticalSliceTests(unittest.TestCase):
    def run_route(self, route):
        payload_hex = FIXTURE.read_bytes().hex()
        input_text = "\n".join(
            [
                json.dumps({"request_id": "one", "payload_hex": payload_hex}),
                '{"request_id":"bad","payload_hex":"0"}',
                json.dumps({"request_id": "two", "payload_hex": payload_hex}),
            ]
        ) + "\n"
        result = subprocess.run(
            [str(RUNNER), "--route", route, "--request-id-prefix", "s7"],
            input=input_text,
            text=True,
            capture_output=True,
            cwd=ROOT,
            check=False,
        )
        self.assertEqual(result.returncode, 0, result.stderr)
        return [json.loads(line) for line in result.stdout.splitlines()]

    def test_one_output_per_input_in_order_for_all_routes(self):
        for route in ("b0", "b1", "b2"):
            with self.subTest(route=route):
                rows = self.run_route(route)
                self.assertEqual(len(rows), 3)
                self.assertEqual([row["request_id"] for row in rows], ["s7-one", "s7-2", "s7-two"])
                self.assertTrue(rows[0]["ok"])
                self.assertFalse(rows[1]["ok"])
                self.assertEqual(rows[1]["error"]["code"], "INVALID_HEX")
                self.assertTrue(rows[2]["ok"])
                if route == "b2":
                    self.assertEqual(rows[0]["observations"]["events"], 6)


if __name__ == "__main__":
    unittest.main()
