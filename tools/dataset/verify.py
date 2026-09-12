import argparse
import hashlib
import json
from pathlib import Path

from .generate import canonical_json_bytes
from .oracle import oracle_bytes

ERROR_CODES = {"TRUNCATED", "BAD_MAGIC", "UNSUPPORTED_VERSION", "LENGTH_MISMATCH", "RANGE_VIOLATION", "UNSUPPORTED_PROTOCOL", "CHECKSUM_FAILURE"}
KINDS = {"normalized_telemetry", "rejection", "execution_failure"}


def _load_json(path: Path) -> dict[str, object]:
    try:
        value = json.loads(path.read_text(encoding="utf-8"))
    except (OSError, json.JSONDecodeError) as exc:
        raise ValueError(f"invalid JSON: {path}") from exc
    if not isinstance(value, dict):
        raise ValueError(f"JSON object required: {path}")
    return value


def _validate_expected(expected: dict[str, object]) -> None:
    required = {"fixture_id", "payload_class", "validity_class", "source_seed", "schema_version", "sha256", "expected"}
    if set(expected) < required:
        raise ValueError(f"expected result missing fields: {expected.get('fixture_id')}")
    if expected["payload_class"] not in (105, 249, 501):
        raise ValueError("invalid payload class")
    if expected["validity_class"] not in ("valid", "invalid", "edge_complex"):
        raise ValueError("invalid validity class")
    if expected["schema_version"] != "1.0" or not isinstance(expected["expected"], dict):
        raise ValueError("invalid expected result schema")
    if expected["expected"].get("kind") not in KINDS:
        raise ValueError("invalid expected result kind")
    if expected["expected"].get("kind") == "rejection" and expected["expected"].get("code") not in ERROR_CODES:
        raise ValueError("invalid rejection code")


def verify_fixture(path: Path, expected: dict[str, object]) -> None:
    payload = path.read_bytes()
    expected_size = expected["payload_class"]
    if expected["validity_class"] == "invalid":
        if len(payload) > expected_size:
            raise ValueError(f"wrong payload size: {path.name}")
    elif len(payload) != expected_size:
        raise ValueError(f"wrong payload size: {path.name}")
    if hashlib.sha256(payload).hexdigest() != expected["sha256"]:
        raise ValueError(f"wrong digest: {path.name}")
    if oracle_bytes(payload) != expected["expected"]:
        raise ValueError(f"oracle mismatch: {path.name}")


def verify_dataset(dataset: Path) -> None:
    manifest = _load_json(dataset / "manifest.json")
    manifest_bytes = canonical_json_bytes(manifest)
    recorded_digest = (dataset / "manifest.sha256").read_text(encoding="ascii").strip()
    if hashlib.sha256(manifest_bytes).hexdigest() != recorded_digest:
        raise ValueError("manifest digest mismatch")
    workload = manifest.get("workload")
    fixtures = manifest.get("fixtures")
    if not isinstance(workload, dict) or not isinstance(fixtures, list):
        raise ValueError("manifest workload or fixtures missing")
    fixture_ids = [item["fixture_id"] for item in fixtures]
    if fixture_ids != sorted(fixture_ids):
        raise ValueError("fixtures are not lexicographically ordered")
    expected_lines = (dataset / "expected-results.jsonl").read_text(encoding="utf-8").splitlines()
    if len(expected_lines) != len(fixtures):
        raise ValueError("expected result count mismatch")
    seen = set()
    category_counts = {"valid": 0, "invalid": 0, "edge_complex": 0}
    for line, metadata in zip(expected_lines, fixtures):
        expected = json.loads(line)
        _validate_expected(expected)
        if expected["fixture_id"] != metadata["fixture_id"] or expected["fixture_id"] in seen:
            raise ValueError("fixture ordering or duplicate mismatch")
        seen.add(expected["fixture_id"])
        category_counts[expected["validity_class"]] += 1
        path = dataset / metadata["filename"]
        if not path.exists():
            raise ValueError(f"missing fixture: {path}")
        verify_fixture(path, expected)
    if category_counts != {key: workload[key] for key in category_counts}:
        raise ValueError("workload counts mismatch")
    actual_files = {path.relative_to(dataset).as_posix() for path in (dataset / "fixtures").glob("*.bin")}
    listed_files = {metadata["filename"] for metadata in fixtures}
    if actual_files != listed_files:
        raise ValueError("fixture file set mismatch")


def main() -> int:
    parser = argparse.ArgumentParser(description="Verify a PRISM telemetry dataset.")
    parser.add_argument("--dataset", type=Path, required=True)
    args = parser.parse_args()
    try:
        verify_dataset(args.dataset)
    except (OSError, ValueError, json.JSONDecodeError) as exc:
        parser.error(str(exc))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
