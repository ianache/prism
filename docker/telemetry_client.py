import fnmatch
import hashlib
import json
import os
import socket
import ssl
import sys
from pathlib import Path


def frame_paths(root: Path, pattern: str) -> list[Path]:
    return sorted(
        path for path in root.rglob("*") if path.is_file() and fnmatch.fnmatch(path.name, pattern)
    )


def iter_stream_payloads(path: Path):
    with path.open(encoding="utf-8") as handle:
        for line_number, line in enumerate(handle, start=1):
            if not line.strip():
                continue
            try:
                record = json.loads(line)
                payload_hex = record["payload_hex"]
                if not isinstance(payload_hex, str):
                    raise ValueError("payload_hex must be a string")
                yield bytes.fromhex(payload_hex)
            except (KeyError, TypeError, ValueError, json.JSONDecodeError) as error:
                raise ValueError(f"invalid stream at line {line_number}: {error}") from error


def main() -> int:
    root = Path(os.environ.get("PRISM_INPUT_PATH", "/data/frames"))
    pattern = os.environ.get("PRISM_INPUT_GLOB", "*.bin")
    target = int(os.environ.get("PRISM_FRAME_TARGET", "100000"))
    prefix = os.environ.get("PRISM_REQUEST_PREFIX", "telemetry")
    allow_cycle = os.environ.get("PRISM_ALLOW_CYCLE", "false").lower() == "true"
    batch_size = int(os.environ.get("PRISM_BATCH_SIZE", "1"))
    stream_path = os.environ.get("PRISM_INPUT_STREAM", "").strip()
    if target <= 0:
        raise ValueError("PRISM_FRAME_TARGET debe ser positivo")
    if batch_size <= 0:
        raise ValueError("PRISM_BATCH_SIZE debe ser positivo")
    paths = [] if stream_path else frame_paths(root, pattern)
    if not stream_path and not paths:
        raise ValueError("dataset vacío")
    if stream_path and not Path(stream_path).is_file():
        raise ValueError(f"stream inexistente: {stream_path}")
    if not stream_path and len(paths) < target and not allow_cycle:
        raise ValueError(f"dataset insuficiente: {len(paths)} archivos para objetivo {target}")

    host = os.environ.get("PRISM_SERVER_HOST", "server")
    port = int(os.environ.get("PRISM_SERVER_PORT", "9000"))
    cafile = os.environ.get("PRISM_TLS_CERT_FILE", "/run/secrets/server-cert.pem")
    token_file = os.environ.get("PRISM_AUTH_TOKEN_FILE", "/run/secrets/auth-token.txt")
    with open(token_file, encoding="utf-8") as handle:
        token = handle.read().strip()

    digest = hashlib.sha256()
    context = ssl.create_default_context(cafile=cafile)
    sent = 0
    payloads = iter_stream_payloads(Path(stream_path)) if stream_path else None
    with socket.create_connection((host, port), timeout=10) as raw:
        with context.wrap_socket(raw, server_hostname="localhost") as tls:
            reader = tls.makefile("rb")
            for batch_start in range(0, target, batch_size):
                requests = []
                for offset in range(min(batch_size, target - batch_start)):
                    index = batch_start + offset + 1
                    if payloads is not None:
                        try:
                            payload = next(payloads)
                        except StopIteration as error:
                            raise ValueError(
                                f"stream insuficiente: {sent} registros para objetivo {target}"
                            ) from error
                    else:
                        path = paths[(index - 1) % len(paths)]
                        payload = path.read_bytes()
                    digest.update(payload)
                    request_id = f"{prefix}-{index:06d}"
                    requests.append(request_id)
                    request = {"request_id": request_id, "payload_hex": payload.hex(), "auth_token": token}
                    tls.sendall((json.dumps(request) + "\n").encode("utf-8"))
                for request_id in requests:
                    response = json.loads(reader.readline())
                    response_id = response.get("request_id", "")
                    if (response_id != request_id and not response_id.endswith(f"-{request_id}")) or response.get("ok") is not True:
                        raise RuntimeError(f"respuesta inválida para {request_id}: {response}")
                    sent += 1

    print(json.dumps({"marker": "S15_TELEMETRY_OK", "frames_sent": sent, "frames_ok": sent, "dataset_sha256": digest.hexdigest(), "synthetic_cycle": allow_cycle}))
    return 0


if __name__ == "__main__":
    try:
        raise SystemExit(main())
    except Exception as error:
        print(f"telemetry client failed: {error}", file=sys.stderr)
        raise SystemExit(1)
