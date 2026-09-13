import json
import socket
import subprocess
import tempfile
import unittest
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
RUNNER = ROOT / "target" / "release" / "prism-run.exe"


class S11AuthenticationTests(unittest.TestCase):
    def test_token_is_required_per_line_and_connection_survives(self):
        with tempfile.TemporaryDirectory() as directory:
            token = Path(directory) / "token.txt"
            token.write_text("secret\n", encoding="utf-8")
            process = subprocess.Popen(
                [str(RUNNER), "--route", "b0", "--listen", "127.0.0.1:0", "--auth-token-file", str(token)],
                cwd=ROOT, stdout=subprocess.DEVNULL, stderr=subprocess.PIPE,
            )
            try:
                ready = process.stderr.readline().decode().strip()
                address = ready.removeprefix("READY ")
                host, port = address.rsplit(":", 1)
                with socket.create_connection((host, int(port)), timeout=5) as client:
                    reader = client.makefile("rb")
                    client.sendall(b'{"request_id":"bad","payload_hex":"00"}\n')
                    bad = json.loads(reader.readline().decode())
                    self.assertEqual(bad["error"]["code"], "AUTHENTICATION_FAILED")
                    client.sendall(b'{"request_id":"good","payload_hex":"00","auth_token":"secret"}\n')
                    good = json.loads(reader.readline().decode())
                    self.assertTrue(good["ok"])
                    self.assertNotIn("secret", json.dumps(bad))
                    reader.close()
            finally:
                process.terminate()
                process.wait(timeout=5)
                process.stderr.close()


if __name__ == "__main__":
    unittest.main()
