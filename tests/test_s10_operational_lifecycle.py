import json
import socket
import subprocess
import tempfile
import time
import unittest
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
RUNNER = ROOT / "target" / "release" / "prism-run.exe"


def read_line(sock):
    data = bytearray()
    while not data.endswith(b"\n"):
        chunk = sock.recv(4096)
        if not chunk:
            break
        data.extend(chunk)
    return json.loads(data.decode())


class S10LifecycleTests(unittest.TestCase):
    def launch(self, sentinel, timeout):
        process = subprocess.Popen(
            [str(RUNNER), "--route", "b0", "--listen", "127.0.0.1:0", "--workers", "1", "--connection-queue", "0", "--shutdown-file", str(sentinel), "--drain-timeout-ms", str(timeout)],
            cwd=ROOT,
            stdin=subprocess.DEVNULL,
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
        )
        states = [process.stderr.readline().decode().strip(), process.stderr.readline().decode().strip()]
        host, port = states[-1].split()[-1].rsplit(":", 1)
        return process, host, int(port), states

    def stop(self, process):
        if process.poll() is None:
            process.terminate()
        process.wait(timeout=5)
        process.stdout.close()
        process.stderr.close()

    def test_draining_rejects_new_connection_and_stops_after_eof(self):
        with tempfile.TemporaryDirectory() as directory:
            sentinel = Path(directory) / "shutdown.flag"
            process, host, port, states = self.launch(sentinel, 1000)
            busy = socket.create_connection((host, port), timeout=5)
            rejected = None
            try:
                busy.sendall(b'{}\n')
                self.assertIsNotNone(busy.recv(4096))
                sentinel.touch()
                self.assertEqual(process.stderr.readline().decode().strip(), "DRAINING")
                rejected = socket.create_connection((host, port), timeout=5)
                self.assertEqual(read_line(rejected)["error"]["code"], "SERVER_DRAINING")
                rejected.close()
                busy.close()
                self.assertEqual(process.stderr.readline().decode().strip(), "STOPPED")
                self.assertEqual(process.wait(timeout=5), 0)
            finally:
                busy.close()
                if rejected is not None:
                    rejected.close()
                self.stop(process)

    def test_idle_connection_is_closed_by_drain_timeout(self):
        with tempfile.TemporaryDirectory() as directory:
            sentinel = Path(directory) / "shutdown.flag"
            process, host, port, _ = self.launch(sentinel, 50)
            idle = socket.create_connection((host, port), timeout=5)
            started = time.monotonic()
            try:
                sentinel.touch()
                self.assertEqual(process.stderr.readline().decode().strip(), "DRAINING")
                self.assertEqual(process.stderr.readline().decode().strip(), "STOPPED")
                self.assertEqual(process.wait(timeout=5), 0)
                self.assertLess(time.monotonic() - started, 2)
            finally:
                idle.close()
                self.stop(process)


if __name__ == "__main__":
    unittest.main()
