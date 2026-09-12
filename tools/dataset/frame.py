import struct

from .checksum import crc32c
from .model import Rejection, SensorRecord, TelemetryFrame

MAGIC = b"PR"
VERSION = 1
HEADER_SIZE = 41
CHECKSUM_SIZE = 4
PAYLOAD_CLASSES = (105, 249, 501)
SENSOR_KINDS = (1, 2, 3)


def payload_class(sensor_area_len: int) -> int:
    value = HEADER_SIZE + sensor_area_len + CHECKSUM_SIZE
    if value not in PAYLOAD_CLASSES:
        raise ValueError("unsupported payload class")
    return value


def _range_check(frame: TelemetryFrame) -> None:
    ranges = (
        ("device_id", frame.device_id, 1, 999999999999999),
        ("timestamp_unix_s", frame.timestamp_unix_s, 0, 4102444800),
        ("latitude_e7", frame.latitude_e7, -900000000, 900000000),
        ("longitude_e7", frame.longitude_e7, -1800000000, 1800000000),
        ("speed_cm_per_s", frame.speed_cm_per_s, 0, 50000),
        ("heading_cdeg", frame.heading_cdeg, 0, 35999),
        ("ignition", frame.ignition, 0, 1),
        ("battery_mv", frame.battery_mv, 0, 60000),
        ("flags", frame.flags, 0, 255),
        ("protocol", frame.protocol, 0, 255),
    )
    if any(not isinstance(value, int) for _, value, _, _ in ranges):
        raise ValueError("all frame fields must be integers")
    for name, value, minimum, maximum in ranges:
        if not minimum <= value <= maximum:
            raise ValueError(f"{name} out of range")
    if frame.protocol != 1:
        raise ValueError("unsupported protocol")
    if frame.payload_class not in PAYLOAD_CLASSES:
        raise ValueError("unsupported payload class")
    area_len = frame.payload_class - HEADER_SIZE - CHECKSUM_SIZE
    if len(frame.sensors) > area_len // 6:
        raise ValueError("too many sensors")
    ids = [sensor.id for sensor in frame.sensors]
    if ids != sorted(set(ids)):
        raise ValueError("sensor ids must be strictly increasing")
    for sensor in frame.sensors:
        if sensor.kind not in SENSOR_KINDS or not 0 <= sensor.id <= 255:
            raise ValueError("sensor out of range")
        if not -2147483648 <= sensor.value <= 2147483647:
            raise ValueError("sensor value out of range")


def encode_frame(frame: TelemetryFrame) -> bytes:
    _range_check(frame)
    area_len = frame.payload_class - HEADER_SIZE - CHECKSUM_SIZE
    body = bytearray(frame.payload_class)
    body[0:2] = MAGIC
    body[2] = VERSION
    body[3] = frame.flags
    struct.pack_into("<H", body, 4, frame.payload_class)
    body[6] = frame.protocol
    struct.pack_into("<Q", body, 7, frame.device_id)
    struct.pack_into("<Q", body, 15, frame.timestamp_unix_s)
    struct.pack_into("<i", body, 23, frame.latitude_e7)
    struct.pack_into("<i", body, 27, frame.longitude_e7)
    struct.pack_into("<H", body, 31, frame.speed_cm_per_s)
    struct.pack_into("<H", body, 33, frame.heading_cdeg)
    body[35] = frame.ignition
    struct.pack_into("<H", body, 36, frame.battery_mv)
    body[38] = len(frame.sensors)
    struct.pack_into("<H", body, 39, area_len)
    for index, sensor in enumerate(frame.sensors):
        offset = HEADER_SIZE + index * 6
        body[offset] = sensor.id
        body[offset + 1] = sensor.kind
        struct.pack_into("<i", body, offset + 2, sensor.value)
    struct.pack_into("<I", body, frame.payload_class - CHECKSUM_SIZE, crc32c(bytes(body[:-4])))
    return bytes(body)


def _reject(code: str, context: dict[str, int | str]) -> tuple[None, Rejection]:
    return None, Rejection(code, "F1_VALIDATE", context)


def decode_frame(payload: bytes) -> tuple[TelemetryFrame | None, Rejection | None]:
    if len(payload) < min(PAYLOAD_CLASSES):
        return _reject("TRUNCATED", {"actual_length": len(payload)})
    if payload[0:2] != MAGIC:
        return _reject("BAD_MAGIC", {"actual_magic": payload[0:2].hex()})
    if payload[2] != VERSION:
        return _reject("UNSUPPORTED_VERSION", {"actual_version": payload[2]})
    declared_length = struct.unpack_from("<H", payload, 4)[0]
    if declared_length != len(payload) or declared_length not in PAYLOAD_CLASSES:
        return _reject("LENGTH_MISMATCH", {"declared_length": declared_length, "actual_length": len(payload)})
    if payload[6] != 1:
        return _reject("UNSUPPORTED_PROTOCOL", {"protocol": payload[6]})
    device_id = struct.unpack_from("<Q", payload, 7)[0]
    timestamp = struct.unpack_from("<Q", payload, 15)[0]
    latitude = struct.unpack_from("<i", payload, 23)[0]
    longitude = struct.unpack_from("<i", payload, 27)[0]
    speed = struct.unpack_from("<H", payload, 31)[0]
    heading = struct.unpack_from("<H", payload, 33)[0]
    ignition = payload[35]
    battery = struct.unpack_from("<H", payload, 36)[0]
    sensor_count = payload[38]
    area_len = struct.unpack_from("<H", payload, 39)[0]
    expected_area_len = len(payload) - HEADER_SIZE - CHECKSUM_SIZE
    scalar_ranges = (
        ("device_id", device_id, 1, 999999999999999),
        ("timestamp_unix_s", timestamp, 0, 4102444800),
        ("latitude_e7", latitude, -900000000, 900000000),
        ("longitude_e7", longitude, -1800000000, 1800000000),
        ("speed_cm_per_s", speed, 0, 50000),
        ("heading_cdeg", heading, 0, 35999),
        ("ignition", ignition, 0, 1),
        ("battery_mv", battery, 0, 60000),
    )
    if area_len != expected_area_len or area_len % 6 or sensor_count > area_len // 6:
        return _reject("RANGE_VIOLATION", {"sensor_area_len": area_len, "sensor_count": sensor_count})
    for name, value, minimum, maximum in scalar_ranges:
        if not minimum <= value <= maximum:
            return _reject("RANGE_VIOLATION", {name: value})
    sensors = []
    previous_id = -1
    for index in range(sensor_count):
        offset = HEADER_SIZE + index * 6
        sensor_id, kind = payload[offset], payload[offset + 1]
        value = struct.unpack_from("<i", payload, offset + 2)[0]
        if sensor_id <= previous_id or kind not in SENSOR_KINDS:
            return _reject("RANGE_VIOLATION", {"sensor_id": sensor_id, "sensor_kind": kind})
        previous_id = sensor_id
        sensors.append(SensorRecord(sensor_id, kind, value))
    padding_start = HEADER_SIZE + sensor_count * 6
    if any(payload[index] != 0 for index in range(padding_start, HEADER_SIZE + area_len)):
        return _reject("RANGE_VIOLATION", {"padding": "nonzero"})
    expected_crc = struct.unpack_from("<I", payload, len(payload) - CHECKSUM_SIZE)[0]
    actual_crc = crc32c(payload[:-CHECKSUM_SIZE])
    if expected_crc != actual_crc:
        return _reject("CHECKSUM_FAILURE", {"expected_crc32c": expected_crc, "actual_crc32c": actual_crc})
    frame = TelemetryFrame(
        device_id, timestamp, latitude, longitude, speed, heading, ignition, battery,
        tuple(sensors), payload[3], 1, len(payload)
    )
    return frame, None


def mutate_frame(payload: bytes, mutation: str) -> bytes:
    value = bytearray(payload)
    if mutation == "truncation":
        return bytes(value[:-1])
    if mutation == "bad_magic":
        value[0] ^= 0xFF
    elif mutation == "bad_version":
        value[2] = 2
    elif mutation == "length_mismatch":
        struct.pack_into("<H", value, 4, len(value) - 1)
    elif mutation == "range_violation":
        value[35] = 2
    elif mutation == "unsupported_protocol":
        value[6] = 2
    elif mutation == "checksum_failure":
        value[-1] ^= 0xFF
    else:
        raise ValueError(f"unknown mutation: {mutation}")
    return bytes(value)
