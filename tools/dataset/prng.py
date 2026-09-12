MASK64 = (1 << 64) - 1


class SplitMix64:
    def __init__(self, seed: int):
        if not 0 <= seed <= MASK64:
            raise ValueError("seed must fit uint64")
        self.state = seed

    def next_u64(self) -> int:
        self.state = (self.state + 0x9E3779B97F4A7C15) & MASK64
        value = self.state
        value = ((value ^ (value >> 30)) * 0xBF58476D1CE4E5B9) & MASK64
        value = ((value ^ (value >> 27)) * 0x94D049BB133111EB) & MASK64
        return (value ^ (value >> 31)) & MASK64

    def next_bounded(self, bound: int) -> int:
        if bound <= 0:
            raise ValueError("bound must be positive")
        return self.next_u64() % bound
