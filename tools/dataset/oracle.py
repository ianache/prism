import argparse

from .model import Rejection


def oracle_bytes(payload: bytes, rule_table_path=None) -> dict[str, object]:
    if not payload:
        return Rejection("TRUNCATED", "F1_VALIDATE", {"actual_length": 0}).as_dict()
    return Rejection("TRUNCATED", "F1_VALIDATE", {"actual_length": len(payload)}).as_dict()


def main() -> int:
    parser = argparse.ArgumentParser(description="Run the PRISM telemetry oracle.")
    parser.add_argument("--input", help="fixture file or directory")
    parser.add_argument("--output", help="canonical JSONL output path")
    parser.parse_args()
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
