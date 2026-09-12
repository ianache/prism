import subprocess
import sys
import tempfile
import unittest
from pathlib import Path


class IntegrationTests(unittest.TestCase):
    def test_generate_oracle_verify_in_temp_directory(self) -> None:
        repository_dataset = Path("datasets/manifest.json").read_bytes()
        with tempfile.TemporaryDirectory() as root:
            dataset = Path(root) / "dataset"
            generate = [sys.executable, "-m", "tools.dataset.generate", "--output", str(dataset), "--seed", "0x505249534D5F5631", "--count", "30"]
            oracle = [sys.executable, "-m", "tools.dataset.oracle", "--input", str(dataset / "fixtures"), "--output", str(dataset / "oracle-results.jsonl")]
            verify = [sys.executable, "-m", "tools.dataset.verify", "--dataset", str(dataset)]
            self.assertEqual(subprocess.run(generate, check=False).returncode, 0)
            self.assertEqual(subprocess.run(oracle, check=False).returncode, 0)
            self.assertEqual(subprocess.run(verify, check=False).returncode, 0)
        self.assertEqual(Path("datasets/manifest.json").read_bytes(), repository_dataset)


if __name__ == "__main__":
    unittest.main()
