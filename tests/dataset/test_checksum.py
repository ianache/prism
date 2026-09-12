import unittest

from tools.dataset.checksum import crc32c


class Crc32cTests(unittest.TestCase):
    def test_empty(self) -> None:
        self.assertEqual(crc32c(b""), 0)

    def test_standard_vector(self) -> None:
        self.assertEqual(crc32c(b"123456789"), 0xE3069283)


if __name__ == "__main__":
    unittest.main()
