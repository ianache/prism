import json
import socket
import subprocess
import unittest
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
RUNNER = ROOT / "target" / "release" / "prism-run.exe"
FIXTURE = next((ROOT / "tests" / "fixtures" / "p0-smoke" / "fixtures").glob("*.bin"))


def receive_line(sock):
    data = bytearray()
    while not data.endswith(b"\n"):
        chunk = sock.recv(4096)
        if not chunk:
            break
        data.extend(chunk)
    return json.loads(data.decode())


class S8TcpTransportTests(unittest.TestCase):
    def test_persistent_connection_and_second_connection_for_all_routes(self):
        payload_hex = FIXTURE.read_bytes().hex()
        for route in ("b0", "b1", "b2"):
            with self.subTest(route=route):
                process = subprocess.Popen(
                    [str(RUNNER), "--route", route, "--listen", "127.0.0.1:0", "--request-id-prefix", "tcp"],
                    cwd=ROOT,
                    stdin=subprocess.DEVNULL,
                    stdout=subprocess.PIPE,
                    stderr=subprocess.PIPE,
                )
                try:
                    address = process.stderr.readline().decode().strip().split()[-1]
                    host, port = address.rsplit(":", 1)
                    with socket.create_connection((host, int(port)), timeout=5) as client:
                        messages = [
                            {"request_id": "one", "payload_hex": payload_hex},
                            {"request_id": "bad", "payload_hex": "0"},
                            {"request_id": "two", "payload_hex": payload_hex},
                        ]
                        client.sendall(("\n".join(json.dumps(item) for item in messages) + "\n").encode())
                        rows = [receive_line(client) for _ in messages]
                        self.assertEqual([row["request_id"] for row in rows], ["tcp-one", "tcp-2", "tcp-two"])
                        self.assertFalse(rows[1]["ok"])
                        self.assertEqual(rows[1]["error"]["code"], "INVALID_HEX")
                        if route == "b2":
                            self.assertEqual(rows[0]["observations"]["events"], 6)
                    with socket.create_connection((host, int(port)), timeout=5) as second:
                        second.sendall((json.dumps({"request_id": "again", "payload_hex": payload_hex}) + "\n").encode())
                        self.assertEqual(receive_line(second)["request_id"], "tcp-again")
                finally:
                    process.terminate()
                    process.wait(timeout=5)
                    process.stdout.close()
                    process.stderr.close()

    def test_oversized_line_closes_only_current_connection(self):
        process = subprocess.Popen(
            [str(RUNNER), "--route", "b0", "--listen", "127.0.0.1:0"],
            cwd=ROOT,
            stdin=subprocess.DEVNULL,
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
        )
        try:
            host, port = process.stderr.readline().decode().strip().split()[-1].rsplit(":", 1)
            with socket.create_connection((host, int(port)), timeout=5) as client:
                client.sendall((b"x" * (64 * 1024 + 1)) + b"\n")
                self.assertEqual(receive_line(client)["error"]["code"], "LINE_TOO_LARGE")
                self.assertEqual(client.recv(1), b"")
            self.assertIsNone(process.poll())
        finally:
            process.terminate()
            process.wait(timeout=5)
            process.stdout.close()
            process.stderr.close()


if __name__ == "__main__":
    unittest.main()
