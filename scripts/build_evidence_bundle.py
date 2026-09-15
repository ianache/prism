import argparse
import json
import os
import sys
import tempfile
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))

from tools.evidence_bundle import CASE_NAMES, validate_bundle


def _case_path(input_dir: Path, name: str) -> Path:
    return input_dir / (name.replace(" ", "-") + ".json")


def build_bundle(input_dir: Path, output: Path, source: str, commit: str, digest: str) -> dict:
    cases = []
    for name in CASE_NAMES:
        path = _case_path(input_dir, name)
        if not path.exists() and name == "tcp 100000" and (input_dir / "result.json").exists():
            path = input_dir / "result.json"
        if not path.exists() and name == "tcp smoke" and (input_dir / "smoke.json").exists():
            path = input_dir / "smoke.json"
        if path.exists():
            value = json.loads(path.read_text(encoding="utf-8"))
            if not isinstance(value, dict):
                raise ValueError(f"case must be an object: {path.name}")
            case = {key: value[key] for key in value if key not in {"payload", "payload_hex", "token", "credentials"}}
            case.setdefault("name", name)
            case.setdefault("protocol", "tcp")
            case.setdefault("status", "PASS" if case.get("frames_sent") == case.get("frames_ok") else "FAIL")
            if name == "tcp 100000" and case.get("frames_sent") != 100000:
                case["status"] = "FAIL"
                case.setdefault("diagnostic", "comparable TCP gate did not report 100000/100000")
        else:
            case = {"name": name, "protocol": name.split()[0], "status": "NO EJECUTADA", "reason": f"evidence case not present: {name}"}
        cases.append(case)
    bundle = {"schema_version": "s24.v1", "source": source, "commit": commit, "digest": digest, "cases": cases}
    errors = validate_bundle(bundle)
    if errors:
        raise ValueError("invalid evidence bundle: " + "; ".join(errors))
    output.parent.mkdir(parents=True, exist_ok=True)
    fd, temp_name = tempfile.mkstemp(prefix=".evidence-bundle-", dir=output.parent)
    try:
        with os.fdopen(fd, "w", encoding="utf-8", newline="\n") as handle:
            json.dump(bundle, handle, indent=2, sort_keys=True)
            handle.write("\n")
        os.replace(temp_name, output)
    except Exception:
        try:
            os.unlink(temp_name)
        except FileNotFoundError:
            pass
        raise
    return bundle


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--input-dir", type=Path, required=True)
    parser.add_argument("--output", type=Path, required=True)
    parser.add_argument("--source", choices=("ci", "wsl2"), required=True)
    parser.add_argument("--commit", required=True)
    parser.add_argument("--digest", required=True)
    args = parser.parse_args()
    build_bundle(args.input_dir, args.output, args.source, args.commit, args.digest)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
