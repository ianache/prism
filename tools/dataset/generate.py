import argparse
from pathlib import Path


def generate_dataset(output: Path, seed: int, count: int, replace: bool = False) -> dict[str, object]:
    if count < 0:
        raise ValueError("count must be non-negative")
    return {"output": str(output), "seed": seed, "count": count, "replace": replace}


def main() -> int:
    parser = argparse.ArgumentParser(description="Generate PRISM telemetry fixtures.")
    parser.add_argument("--output", type=Path, required=True)
    parser.add_argument("--seed", type=lambda value: int(value, 0), required=True)
    parser.add_argument("--count", type=int, required=True)
    parser.add_argument("--replace", action="store_true")
    parser.parse_args()
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
