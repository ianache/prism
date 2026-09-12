import argparse
import json
from pathlib import Path

from .frame import decode_frame
from .model import TelemetryFrame
from .rules import DEFAULT_RULE_TABLE, evaluate_rules, load_rule_table


def _normalized(frame: TelemetryFrame, result: dict[str, str]) -> dict[str, object]:
    return {
        "kind": "normalized_telemetry",
        "schema_version": "1.0",
        "device_id": frame.device_id,
        "timestamp_unix_s": frame.timestamp_unix_s,
        "latitude_e7": frame.latitude_e7,
        "longitude_e7": frame.longitude_e7,
        "speed_cm_per_s": frame.speed_cm_per_s,
        "heading_cdeg": frame.heading_cdeg,
        "ignition": frame.ignition,
        "battery_mv": frame.battery_mv,
        "sensors": [{"id": item.id, "kind": item.kind, "value": item.value} for item in frame.sensors],
        "classification": result["label"],
        "severity": result["severity"],
        "route": result["route"],
    }


def oracle_bytes(payload: bytes, rule_table_path: Path = DEFAULT_RULE_TABLE) -> dict[str, object]:
    try:
        frame, rejection = decode_frame(payload)
        if rejection is not None:
            return rejection.as_dict()
        assert frame is not None
        return _normalized(frame, evaluate_rules(frame, load_rule_table(rule_table_path)))
    except Exception as exc:
        return {"kind": "execution_failure", "schema_version": "1.0", "error_type": type(exc).__name__, "message": str(exc)}


def _iter_input(path: Path):
    if path.is_file():
        yield path
    else:
        yield from sorted(path.glob("*.bin"))


def main() -> int:
    parser = argparse.ArgumentParser(description="Run the PRISM telemetry oracle.")
    parser.add_argument("--input", type=Path, required=True)
    parser.add_argument("--output", type=Path, required=True)
    args = parser.parse_args()
    if args.output.exists():
        parser.error(f"output already exists: {args.output}")
    args.output.parent.mkdir(parents=True, exist_ok=True)
    with args.output.open("w", encoding="utf-8") as stream:
        for path in _iter_input(args.input):
            result = oracle_bytes(path.read_bytes())
            stream.write(json.dumps(result, ensure_ascii=False, sort_keys=True, separators=(",", ":")) + "\n")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
