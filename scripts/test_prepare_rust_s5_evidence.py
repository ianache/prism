import json
import subprocess
import tempfile
import unittest
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
SCRIPT = ROOT / "scripts" / "prepare-rust-s5-evidence.ps1"


class PreflightTests(unittest.TestCase):
    def run_script(self, dataset, raw, manifest):
        return subprocess.run(
            ["powershell", "-NoProfile", "-ExecutionPolicy", "Bypass", "-File", str(SCRIPT),
             "-Dataset", str(dataset), "-RawOutput", str(raw), "-ManifestOutput", str(manifest)],
            capture_output=True, text=True,
        )

    def test_records_frozen_parameters(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            dataset = root / "dataset"
            dataset.mkdir()
            (dataset / "manifest.json").write_text(json.dumps({"dataset_id": "d"}), encoding="utf-8")
            (dataset / "manifest.sha256").write_text("a" * 64, encoding="ascii")
            result = self.run_script(dataset, root / "raw.jsonl", root / "preflight.json")
            self.assertEqual(result.returncode, 0, result.stderr)
            data = json.loads((root / "preflight.json").read_text(encoding="utf-8-sig"))
            self.assertEqual((data["samples"], data["repetitions"], data["duration_seconds"]), (100000, 5, 900))
            self.assertEqual(data["dataset_digest"], "a" * 64)

    def test_rejects_existing_output(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            dataset = root / "dataset"
            dataset.mkdir()
            (dataset / "manifest.json").write_text(json.dumps({"dataset_id": "d"}), encoding="utf-8")
            (dataset / "manifest.sha256").write_text("a" * 64, encoding="ascii")
            raw = root / "raw.jsonl"
            raw.write_text("existing", encoding="utf-8")
            result = self.run_script(dataset, raw, root / "preflight.json")
            self.assertNotEqual(result.returncode, 0)


if __name__ == "__main__":
    unittest.main()
