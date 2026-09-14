"""Check production Compose identity and secret-file readiness without leaking secrets."""

import argparse
import json
import os
import stat
import sys
from pathlib import Path


SECRET_NAMES = ("server-cert.pem", "server-key.pem", "auth-token.txt")


def validate(root: Path, secret_dir: Path, expected_uid: int) -> dict[str, object]:
    overlay = (root / "docker-compose.production.yml").read_text(encoding="utf-8")
    if 'user: "10001:10001"' not in overlay:
        raise ValueError("production overlay is not pinned to UID/GID 10001")
    env_text = (root / ".env.production.example").read_text(encoding="utf-8")
    if "PRISM_ALLOW_CYCLE=false" not in env_text:
        raise ValueError("production defaults must disable synthetic cycling")

    files = []
    owner_checked = hasattr(os, "getuid")
    current_uid = os.getuid() if owner_checked else None
    for name in SECRET_NAMES:
        path = secret_dir / name
        if not path.is_file():
            raise ValueError(f"missing secret file: {name}")
        mode = stat.S_IMODE(path.stat().st_mode)
        if os.name != "nt" and mode & 0o077:
            raise ValueError(f"secret file is too permissive: {name}")
        if owner_checked and path.stat().st_uid not in (0, expected_uid, current_uid):
            raise ValueError(f"unexpected secret owner: {name}")
        files.append({"name": name, "mode": oct(mode), "owner_checked": owner_checked})
    return {
        "marker": "S18_PREFLIGHT_OK",
        "expected_uid": expected_uid,
        "effective_uid": os.getuid() if owner_checked else None,
        "secret_files": files,
        "synthetic_cycle": False,
    }


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--root", type=Path, default=Path("."))
    parser.add_argument("--secret-dir", type=Path, required=True)
    parser.add_argument("--expected-uid", type=int, default=10001)
    parser.add_argument("--output", type=Path)
    args = parser.parse_args()
    try:
        result = validate(args.root.resolve(), args.secret_dir.resolve(), args.expected_uid)
    except (OSError, ValueError) as error:
        print(f"S18_PREFLIGHT_FAILED: {error}", file=sys.stderr)
        return 1
    encoded = json.dumps(result, sort_keys=True)
    if args.output:
        args.output.write_text(encoded + "\n", encoding="utf-8")
    print(encoded)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
