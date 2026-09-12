import argparse
from pathlib import Path


def verify_dataset(dataset: Path) -> None:
    if not dataset.exists():
        raise ValueError(f"dataset does not exist: {dataset}")


def main() -> int:
    parser = argparse.ArgumentParser(description="Verify a PRISM telemetry dataset.")
    parser.add_argument("--dataset", type=Path, required=True)
    parser.parse_args()
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
