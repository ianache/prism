import argparse
import json
import os
import platform
import subprocess
from pathlib import Path


def command(*args):
    return subprocess.run(args, check=True, capture_output=True, text=True, timeout=30).stdout.strip()


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--dataset", type=Path, required=True)
    parser.add_argument("--raw-output", type=Path, required=True)
    parser.add_argument("--manifest-output", type=Path, required=True)
    args = parser.parse_args()
    dataset = args.dataset.resolve(strict=True)
    if args.raw_output.exists():
        raise SystemExit(f"raw output already exists: {args.raw_output}")
    manifest = json.loads((dataset / "manifest.json").read_text(encoding="utf-8"))
    digest = (dataset / "manifest.sha256").read_text(encoding="ascii").strip()
    record = {
        "repository": command("git", "rev-parse", "--show-toplevel"),
        "commit": command("git", "rev-parse", "HEAD"),
        "dataset": str(dataset),
        "dataset_id": manifest["dataset_id"],
        "dataset_digest": digest,
        "scenario": "S5",
        "increment": "S6-p99-root-cause-isolation",
        "protocol_version": "1.1",
        "samples": 100000,
        "repetitions": 5,
        "duration_seconds": 900,
        "concurrency": 1,
        "cargo": command(r"C:\Users\ilver\.cargo\bin\cargo.exe", "--version"),
        "os": platform.platform(),
        "processor_count": os.cpu_count(),
        "output": str(args.raw_output.resolve()),
        "output_available": True,
    }
    args.manifest_output.parent.mkdir(parents=True, exist_ok=True)
    args.manifest_output.write_text(json.dumps(record, indent=2) + "\n", encoding="utf-8")
    print(json.dumps(record, separators=(",", ":")))


if __name__ == "__main__":
    main()
