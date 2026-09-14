"""Validate the repository's production-readiness CI contract."""

import json
import subprocess
import sys
from pathlib import Path


REQUIRED_JOBS = ("rust:", "contracts:", "compose:", "docker-build:")
FORBIDDEN_MARKERS = ("-----BEGIN " + "PRIVATE KEY-----", "-----BEGIN " + "RSA PRIVATE KEY-----")


def tracked_files(root: Path) -> list[Path]:
    result = subprocess.run(
        ["git", "-C", str(root), "ls-files"], capture_output=True, text=True, check=True
    )
    return [root / line for line in result.stdout.splitlines() if line]


def validate(root: Path) -> dict[str, object]:
    workflow_path = root / ".github" / "workflows" / "production-readiness.yml"
    workflow = workflow_path.read_text(encoding="utf-8")
    missing_jobs = [job for job in REQUIRED_JOBS if job not in workflow]
    if missing_jobs:
        raise ValueError(f"missing CI jobs: {missing_jobs}")
    for command in ("cargo test --workspace --release", "docker-compose.production.yml"):
        if command not in workflow:
            raise ValueError(f"missing CI command: {command}")

    violations = []
    for path in tracked_files(root):
        if path.is_file():
            text = path.read_text(encoding="utf-8", errors="ignore")
            if any(marker in text for marker in FORBIDDEN_MARKERS):
                violations.append(path.relative_to(root).as_posix())
        relative = path.relative_to(root).as_posix()
        if relative.startswith("secrets/") or relative in {".env", ".env.production"}:
            violations.append(relative)
    if violations:
        raise ValueError(f"tracked secret material: {sorted(set(violations))}")
    return {"workflow": workflow_path.relative_to(root).as_posix(), "tracked_files": len(tracked_files(root))}


def main() -> int:
    root = Path(sys.argv[1] if len(sys.argv) > 1 else ".").resolve()
    try:
        print(json.dumps({"marker": "S18_CI_CONTRACT_OK", **validate(root)}, sort_keys=True))
    except (OSError, subprocess.CalledProcessError, ValueError) as error:
        print(f"S18_CI_CONTRACT_FAILED: {error}", file=sys.stderr)
        return 1
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
