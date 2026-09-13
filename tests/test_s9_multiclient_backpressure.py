import json
import socket
import subprocess
import time
import unittest
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
RUNNER = ROOT / "target" / "release" / "prism-run.exe"
FIXTURE = next((ROOT / "tests" / "fixtures" / "p0-smoke" / "fixtures").glob("*.bin"))


def read_line(sock):
    data = bytearray()
    while not data.endswith(b"\n"):
        chunk = sock.recv(4096)
        if not chunk:
            break
        data.extend(chunk)
    return json.loads(data.decode())


class S9MultiClientTests(unittest.TestCase):
    def start(self, workers, queue, route="b2"):
        process = subprocess.Popen(
            [str(RUNNER), "--route", route, "--listen", "127.0.0.1:0", "--workers", str(workers), "--connection-queue", str(queue), "--request-id-prefix", "s9"],
            cwd=ROOT,
            stdin=subprocess.DEVNULL,
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
        )
        address = process.stderr.readline().decode().strip().split()[-1]
        host, port = address.rsplit(":", 1)
        return process, host, int(port)

    def stop(self, process):
        process.terminate()
        process.wait(timeout=5)
        process.stdout.close()
        process.stderr.close()

    def test_multiple_clients_keep_individual_order_and_b2_observation(self):
        payload_hex = FIXTURE.read_bytes().hex()
        process, host, port = self.start(2, 2)
        clients = []
        readers = []
        try:
            for index in range(3):
                client = socket.create_connection((host, port), timeout=5)
                clients.append(client)
                readers.append(client.makefile("rb"))
            for index, client in enumerate(clients):
                for suffix in ("a", "b"):
                    client.sendall((json.dumps({"request_id": f"c{index}-{suffix}", "payload_hex": payload_hex}) + "\n").encode())
            completed = []
            for index in range(2):
                rows = [json.loads(readers[index].readline().decode()), json.loads(readers[index].readline().decode())]
                self.assertEqual([row["request_id"] for row in rows], [f"s9-c{index}-a", f"s9-c{index}-b"])
                self.assertTrue(all(row["ok"] for row in rows))
                self.assertEqual(rows[0]["observations"]["events"], 6)
                completed.append(index)
            for index in completed:
                readers[index].close()
                clients[index].close()
            rows = [json.loads(readers[2].readline().decode()), json.loads(readers[2].readline().decode())]
            self.assertEqual([row["request_id"] for row in rows], ["s9-c2-a", "s9-c2-b"])
            self.assertTrue(all(row["ok"] for row in rows))
            self.assertEqual(rows[0]["observations"]["events"], 6)
        finally:
            for client in clients:
                client.close()
            for reader in readers:
                reader.close()
            self.stop(process)

    def test_full_capacity_rejects_and_recovers_after_eof(self):
        payload_hex = FIXTURE.read_bytes().hex()
        process, host, port = self.start(1, 0, route="b0")
        try:
            busy = socket.create_connection((host, port), timeout=5)
            busy.sendall(b"{\"request_id\":\"busy\"")
            time.sleep(0.1)
            rejected = socket.create_connection((host, port), timeout=5)
            self.assertEqual(read_line(rejected)["error"]["code"], "CAPACITY_EXCEEDED")
            rejected.close()
            busy.close()
            recovered = socket.create_connection((host, port), timeout=5)
            recovered.sendall((json.dumps({"request_id": "recovered", "payload_hex": payload_hex}) + "\n").encode())
            self.assertEqual(read_line(recovered)["request_id"], "s9-recovered")
            recovered.close()
            self.assertIsNone(process.poll())
        finally:
            self.stop(process)


if __name__ == "__main__":
    unittest.main()
