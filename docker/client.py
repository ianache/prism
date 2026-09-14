import json
import os
import socket
import ssl


host = os.environ.get("PRISM_SERVER_HOST", "server")
port = int(os.environ.get("PRISM_SERVER_PORT", "9000"))
cafile = os.environ.get("PRISM_TLS_CERT_FILE", "/run/secrets/server-cert.pem")
token_file = os.environ.get("PRISM_AUTH_TOKEN_FILE", "/run/secrets/auth-token.txt")
with open(token_file, encoding="utf-8") as handle:
    token = handle.read().strip()

request = {"request_id": "compose-client", "payload_hex": "00", "auth_token": token}
context = ssl.create_default_context(cafile=cafile)
with socket.create_connection((host, port), timeout=5) as raw:
    with context.wrap_socket(raw, server_hostname="localhost") as tls:
        tls.sendall((json.dumps(request) + "\n").encode("utf-8"))
        response = tls.makefile("rb").readline()
print(response.decode("utf-8").rstrip())
