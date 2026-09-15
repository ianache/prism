import argparse
import json
import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))

from tools.evidence_bundle import validate_bundle


def audit_bundle(path: str | Path) -> str:
    bundle = json.loads(Path(path).read_text(encoding="utf-8"))
    errors = validate_bundle(bundle)
    if errors:
        raise ValueError("invalid evidence bundle: " + "; ".join(errors))
    counts = {"PASS": 0, "FAIL": 0, "NO EJECUTADA": 0}
    for case in bundle["cases"]:
        counts[case["status"]] += 1
    return f"ok pass={counts['PASS']} fail={counts['FAIL']} not_executed={counts['NO EJECUTADA']} frames=100000"


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("bundle", type=Path)
    args = parser.parse_args()
    print(audit_bundle(args.bundle))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
