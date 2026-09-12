import unittest

from tools.dataset.prng import SplitMix64


class SplitMix64Tests(unittest.TestCase):
    def test_known_sequence(self) -> None:
        rng = SplitMix64(0)
        self.assertEqual(rng.next_u64(), 0xE220A8397B1DCDAF)
        self.assertEqual(rng.next_u64(), 0x6E789E6AA1B965F4)

    def test_bounded_values_and_invalid_bound(self) -> None:
        rng = SplitMix64(0x505249534D5F5631)
        values = [rng.next_bounded(17) for _ in range(100)]
        self.assertTrue(all(0 <= value < 17 for value in values))
        with self.assertRaises(ValueError):
            rng.next_bounded(0)

    def test_seed_must_fit_uint64(self) -> None:
        with self.assertRaises(ValueError):
            SplitMix64(-1)
        with self.assertRaises(ValueError):
            SplitMix64(1 << 64)


if __name__ == "__main__":
    unittest.main()
