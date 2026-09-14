"""Generate a sequential, distinct, valid B2 telemetry JSONL corpus."""

import argparse
import hashlib
import json
import struct
from pathlib import Path


def crc32c(data: bytes) -> int:
    crc = 0xFFFFFFFF
    for byte in data:
        crc ^= byte
        for _ in range(8):
            crc = (crc >> 1) ^ (0x82F63B78 if crc & 1 else 0)
    return crc ^ 0xFFFFFFFF


def build_payload(template: bytes, index: int) -> bytes:
    if len(template) not in (105, 249, 501):
        raise ValueError("template must be a supported 105, 249, or 501 byte frame")
    payload = bytearray(template)
    payload[7:15] = struct.pack("<Q", index)
    payload[15:23] = struct.pack("<Q", 1_700_000_000 + index)
    payload[-4:] = struct.pack("<I", crc32c(payload[:-4]))
    return bytes(payload)


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--template", type=Path, required=True)
    parser.add_argument("--output", type=Path, required=True)
    parser.add_argument("--manifest", type=Path, required=True)
    parser.add_argument("--frames", type=int, default=100_000)
    args = parser.parse_args()
    if args.frames <= 0:
        raise SystemExit("--frames must be positive")
    template = args.template.read_bytes()
    args.output.parent.mkdir(parents=True, exist_ok=True)
    digest = hashlib.sha256()
    with args.output.open("w", encoding="utf-8", newline="\n") as handle:
        for index in range(1, args.frames + 1):
            payload = build_payload(template, index)
            digest.update(payload)
            handle.write(json.dumps({"index": index, "payload_hex": payload.hex()}, separators=(",", ":")))
            handle.write("\n")
    manifest = {
        "format": "prism-telemetry-jsonl-v1",
        "frames": args.frames,
        "payload_bytes": len(template),
        "payload_sha256": digest.hexdigest(),
        "distinct": True,
    }
    args.manifest.write_text(json.dumps(manifest, indent=2) + "\n", encoding="utf-8")
    print(json.dumps(manifest, sort_keys=True))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
