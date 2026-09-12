import argparse
import hashlib
import json
import shutil
import tempfile
from pathlib import Path

from .frame import encode_frame, mutate_frame
from .model import SensorRecord, TelemetryFrame
from .oracle import oracle_bytes
from .prng import SplitMix64

SEED = 0x505249534D5F5631
PAYLOAD_CLASSES = (105, 249, 501)
INVALID_MUTATIONS = ("truncation", "bad_magic", "bad_version", "length_mismatch", "range_violation", "unsupported_protocol", "checksum_failure")


def canonical_json_bytes(value: object) -> bytes:
    return (json.dumps(value, ensure_ascii=False, sort_keys=True, separators=(",", ":")) + chr(10)).encode("utf-8")


def fixture_id(index: int, validity_class: str, payload_class: int) -> str:
    return f"p0-{validity_class}-{index:07d}-{payload_class:03d}"


def _sensor_records(rng: SplitMix64, count: int, base: int = 0) -> tuple[SensorRecord, ...]:
    return tuple(SensorRecord(index + 1, (index % 3) + 1, base + rng.next_bounded(100000)) for index in range(count))


def _base_frame(index: int, payload_class: int, rng: SplitMix64) -> TelemetryFrame:
    area_records = (payload_class - 45) // 6
    sensor_count = rng.next_bounded(area_records + 1)
    return TelemetryFrame(100000000000000 + rng.next_bounded(899999999999999), 1700000000 + rng.next_bounded(100000000), -900000000 + rng.next_bounded(1800000001), -1800000000 + rng.next_bounded(3600000001), rng.next_bounded(50001), rng.next_bounded(36000), rng.next_bounded(2), rng.next_bounded(60001), _sensor_records(rng, sensor_count), flags=rng.next_bounded(256), payload_class=payload_class)


def _edge_frame(index: int, payload_class: int, rng: SplitMix64) -> TelemetryFrame:
    frame = _base_frame(index, payload_class, rng)
    case = index % 6
    if case == 0:
        return TelemetryFrame(**{**frame.__dict__, "ignition": 0})
    if case == 1:
        return TelemetryFrame(**{**frame.__dict__, "ignition": 1, "battery_mv": 11000})
    if case == 2:
        return TelemetryFrame(**{**frame.__dict__, "ignition": 1, "speed_cm_per_s": 2778})
    if case == 3:
        return TelemetryFrame(**{**frame.__dict__, "ignition": 1, "sensors": (SensorRecord(1, 1, 85000),)})
    if case == 4:
        return TelemetryFrame(1, 0, -900000000, -1800000000, 0, 0, 0, 0, (), payload_class=payload_class)
    max_sensors = (payload_class - 45) // 6
    return TelemetryFrame(999999999999999, 4102444800, 900000000, 1800000000, 50000, 35999, 1, 60000, _sensor_records(rng, max_sensors), flags=255, payload_class=payload_class)


def generate_case(index: int, validity_class: str, payload_class: int, rng: SplitMix64) -> tuple[str, bytes, dict[str, object]]:
    frame = _edge_frame(index, payload_class, rng) if validity_class == "edge_complex" else _base_frame(index, payload_class, rng)
    payload = encode_frame(frame)
    if validity_class == "invalid":
        payload = mutate_frame(payload, INVALID_MUTATIONS[index % len(INVALID_MUTATIONS)])
    return fixture_id(index, validity_class, payload_class), payload, oracle_bytes(payload)


def _counts(count: int) -> tuple[int, int, int]:
    weights = (80, 15, 5)
    quotas = [count * weight // 100 for weight in weights]
    remainder = count - sum(quotas)
    fractions = [count * weight % 100 for weight in weights]
    for index in sorted(range(len(weights)), key=lambda item: (-fractions[item], item))[:remainder]:
        quotas[index] += 1
    valid, edge_count, invalid = quotas
    return valid, invalid, edge_count


def _write_dataset(temp: Path, seed: int, count: int) -> dict[str, object]:
    valid_count, invalid_count, edge_count = _counts(count)
    categories = (("valid", valid_count), ("edge_complex", edge_count), ("invalid", invalid_count))
    rng = SplitMix64(seed)
    manifest = {"schema_version": "1.0", "dataset_id": "prism.telemetry.p0.v1", "generator_version": "1.0", "oracle_version": "1.0", "seed": f"0x{seed:016X}", "prng": "SplitMix64", "encoding": "raw-binary-plus-canonical-jsonl", "payload_classes": list(PAYLOAD_CLASSES), "workload": {"total": count, "valid": valid_count, "invalid": invalid_count, "edge_complex": edge_count}, "fixture_order": "lexicographic fixture_id", "digest_algorithm": "SHA-256", "expected_results": "expected-results.jsonl"}
    expected_path = temp / "expected-results.jsonl"
    manifest_path = temp / "manifest.json"
    fixtures_path = temp / "fixtures"
    fixtures_path.mkdir(parents=True, exist_ok=True)
    ordered_categories = tuple(sorted(categories))
    with expected_path.open("wb") as expected_stream, manifest_path.open("wb") as manifest_stream:
        manifest_stream.write(b"{")
        for key in sorted(key for key in manifest if key < "fixtures"):
            if manifest_stream.tell() > 1:
                manifest_stream.write(b",")
            manifest_stream.write(json.dumps(key, ensure_ascii=False, separators=(",", ":")).encode("utf-8"))
            manifest_stream.write(b":")
            manifest_stream.write(json.dumps(manifest[key], ensure_ascii=False, sort_keys=True, separators=(",", ":")).encode("utf-8"))
        manifest_stream.write(b',"fixtures":[')
        first_fixture = True
        index = 0
        for category, category_count in ordered_categories:
            for _ in range(category_count):
                payload_class = PAYLOAD_CLASSES[index % len(PAYLOAD_CLASSES)]
                fixture, payload, expected = generate_case(index, category, payload_class, rng)
                path = fixtures_path / f"{fixture}.bin"
                path.write_bytes(payload)
                digest = hashlib.sha256(payload).hexdigest()
                record = {"fixture_id": fixture, "payload_class": payload_class, "validity_class": category, "source_seed": f"0x{seed:016X}", "schema_version": "1.0", "sha256": digest, "expected": expected}
                metadata = {**record, "filename": f"fixtures/{fixture}.bin", "size": len(payload)}
                if not first_fixture:
                    manifest_stream.write(b",")
                first_fixture = False
                manifest_stream.write(json.dumps(metadata, ensure_ascii=False, sort_keys=True, separators=(",", ":")).encode("utf-8"))
                expected_stream.write(canonical_json_bytes(record))
                index += 1
        manifest_stream.write(b"]")
        for key in sorted(key for key in manifest if key > "fixtures"):
            manifest_stream.write(b",")
            manifest_stream.write(json.dumps(key, ensure_ascii=False, separators=(",", ":")).encode("utf-8"))
            manifest_stream.write(b":")
            manifest_stream.write(json.dumps(manifest[key], ensure_ascii=False, sort_keys=True, separators=(",", ":")).encode("utf-8"))
        manifest_stream.write(b"}")
    manifest_bytes = manifest_path.read_bytes()
    manifest_path.write_bytes(manifest_bytes + chr(10).encode("ascii"))
    manifest_bytes += chr(10).encode("ascii")
    (temp / "manifest.sha256").write_text(hashlib.sha256(manifest_bytes).hexdigest() + chr(10), encoding="ascii")
    return {**manifest, "fixtures": []}


def generate_dataset(output: Path, seed: int, count: int, replace: bool = False) -> dict[str, object]:
    if count < 0:
        raise ValueError("count must be non-negative")
    if output.exists() and any(output.iterdir()) and not replace:
        raise ValueError(f"output is not empty: {output}")
    output.parent.mkdir(parents=True, exist_ok=True)
    temp_path = Path(tempfile.mkdtemp(prefix=output.name + ".tmp-", dir=output.parent))
    try:
        manifest = _write_dataset(temp_path, seed, count)
        if output.exists():
            shutil.rmtree(output)
        temp_path.rename(output)
        return manifest
    except Exception:
        shutil.rmtree(temp_path, ignore_errors=True)
        raise


def main() -> int:
    parser = argparse.ArgumentParser(description="Generate PRISM telemetry fixtures.")
    parser.add_argument("--output", type=Path, required=True)
    parser.add_argument("--seed", type=lambda value: int(value, 0), default=SEED)
    parser.add_argument("--count", type=int, default=1000000)
    parser.add_argument("--replace", action="store_true")
    args = parser.parse_args()
    try:
        generate_dataset(args.output, args.seed, args.count, args.replace)
    except ValueError as exc:
        parser.error(str(exc))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
