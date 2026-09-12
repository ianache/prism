POLY = 0x82F63B78


def _table() -> tuple[int, ...]:
    values = []
    for byte in range(256):
        value = byte
        for _ in range(8):
            value = (value >> 1) ^ POLY if value & 1 else value >> 1
        values.append(value & 0xFFFFFFFF)
    return tuple(values)


TABLE = _table()


def crc32c(data: bytes) -> int:
    value = 0xFFFFFFFF
    for byte in data:
        value = (value >> 8) ^ TABLE[(value ^ byte) & 0xFF]
    return (value ^ 0xFFFFFFFF) & 0xFFFFFFFF
