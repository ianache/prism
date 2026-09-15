import json
import os
import socket
import ssl
import sys
import urllib.request


def http_check(host: str, port: int, cafile: str) -> int:
    context = ssl.create_default_context(cafile=cafile)
    for path in ("/healthz", "/readyz"):
        with urllib.request.urlopen(f"https://{host}:{port}{path}", context=context, timeout=3) as response:
            payload = json.loads(response.read())
        if path == "/healthz" and payload.get("status") != "ok":
            return 1
        if path == "/readyz" and payload.get("ready") is not True:
            return 1
    return 0


def main() -> int:
    host, port_text = os.environ.get("PRISM_HEALTH_ADDRESS", "127.0.0.1:9000").rsplit(":", 1)
    cafile = os.environ.get("PRISM_TLS_CERT_FILE", "/run/secrets/server-cert.pem")
    if os.environ.get("PRISM_PROTOCOL", "tcp") == "http":
        return http_check(host, int(port_text), cafile)
    token_file = os.environ.get("PRISM_AUTH_TOKEN_FILE", "/run/secrets/auth-token.txt")
    with open(token_file, encoding="utf-8") as handle:
        token = handle.read().strip()
    request = {"request_id": "compose-healthcheck", "payload_hex": "00", "auth_token": token}
    context = ssl.create_default_context(cafile=cafile)
    with socket.create_connection((host, int(port_text)), timeout=3) as raw:
        with context.wrap_socket(raw, server_hostname="localhost") as tls:
            tls.sendall((json.dumps(request) + "\n").encode("utf-8"))
            response = tls.makefile("rb").readline()
    result = json.loads(response)
    return 0 if result.get("ok") is True and result.get("route") == "b2" else 1


if __name__ == "__main__":
    try:
        raise SystemExit(main())
    except Exception as error:
        print(f"healthcheck failed: {error}", file=sys.stderr)
        raise SystemExit(1)
