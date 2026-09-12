import tempfile
import unittest
from pathlib import Path

from tools.dataset.generate import SEED, generate_dataset


class GenerateTests(unittest.TestCase):
    def test_small_counts_and_classes(self) -> None:
        with tempfile.TemporaryDirectory() as first, tempfile.TemporaryDirectory() as second:
            first_path = Path(first) / "dataset"
            second_path = Path(second) / "dataset"
            first_manifest = generate_dataset(first_path, SEED, 300)
            second_manifest = generate_dataset(second_path, SEED, 300)
            self.assertEqual(first_manifest["workload"], {"total": 300, "valid": 240, "invalid": 15, "edge_complex": 45})
            self.assertEqual(first_manifest, second_manifest)
            first_files = sorted((first_path / "fixtures").glob("*.bin"))
            second_files = sorted((second_path / "fixtures").glob("*.bin"))
            self.assertEqual([path.name for path in first_files], [path.name for path in second_files])
            self.assertEqual([path.read_bytes() for path in first_files], [path.read_bytes() for path in second_files])

    def test_nonempty_output_requires_replace(self) -> None:
        with tempfile.TemporaryDirectory() as root:
            output = Path(root) / "dataset"
            generate_dataset(output, SEED, 3)
            with self.assertRaises(ValueError):
                generate_dataset(output, SEED, 3)
            generate_dataset(output, SEED, 3, replace=True)


if __name__ == "__main__":
    unittest.main()
