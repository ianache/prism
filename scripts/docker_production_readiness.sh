#!/usr/bin/env bash
set -Eeuo pipefail

frames=100000
batch_size=64
evidence_dir=""
while (($#)); do
  case "$1" in
    --frames) frames="$2"; shift 2 ;;
    --batch-size) batch_size="$2"; shift 2 ;;
    --evidence-dir) evidence_dir="$2"; shift 2 ;;
    *) echo "unknown argument: $1" >&2; exit 2 ;;
  esac
done
[[ "$frames" =~ ^[0-9]+$ ]] && (( frames > 0 )) || { echo "frames must be positive" >&2; exit 2; }
[[ "$batch_size" =~ ^[0-9]+$ ]] && (( batch_size > 0 )) || { echo "batch-size must be positive" >&2; exit 2; }

root="$(cd "$(dirname "$0")/.." && pwd)"
runtime_dir="$(mktemp -d)"
secret_dir="$runtime_dir/secrets"
data_dir="$runtime_dir/data"
if [[ -z "$evidence_dir" ]]; then evidence_dir="$runtime_dir/evidence"; fi
mkdir -p "$secret_dir" "$data_dir" "$evidence_dir"
compose=(docker compose --env-file "$runtime_dir/s18.env" -f "$root/docker-compose.yml" -f "$root/docker-compose.production.yml")

cleanup() {
  "${compose[@]}" down --remove-orphans >/dev/null 2>&1 || true
  rm -rf "$runtime_dir"
}
trap cleanup EXIT

openssl req -x509 -newkey rsa:2048 -nodes -days 1 \
  -subj "/CN=localhost" -addext "subjectAltName=DNS:localhost" \
  -keyout "$secret_dir/server-key.pem" -out "$secret_dir/server-cert.pem" >/dev/null 2>&1
printf 's18-ephemeral-token\n' > "$secret_dir/auth-token.txt"
chmod 600 "$secret_dir"/*
export PRISM_TLS_CERT_CONTENT="$(cat "$secret_dir/server-cert.pem")"
export PRISM_TLS_KEY_CONTENT="$(cat "$secret_dir/server-key.pem")"
export PRISM_AUTH_TOKEN_CONTENT="$(cat "$secret_dir/auth-token.txt")"
python3 "$root/scripts/generate_telemetry_stream.py" \
  --template "$root/tests/fixtures/p0-smoke/fixtures/p0-valid-0000060-105.bin" \
  --output "$data_dir/telemetry.jsonl" --manifest "$data_dir/telemetry.json" --frames "$frames" >/dev/null
cat > "$runtime_dir/s18.env" <<EOF
PRISM_CONTAINER_USER=10001:10001
PRISM_PROTOCOL=${PRISM_PROTOCOL:-tcp}
PRISM_TLS_CERT=$secret_dir/server-cert.pem
PRISM_TLS_KEY=$secret_dir/server-key.pem
PRISM_AUTH_TOKEN=$secret_dir/auth-token.txt
PRISM_INPUT_PATH=$data_dir
PRISM_INPUT_STREAM=/data/frames/telemetry.jsonl
PRISM_FRAME_TARGET=$frames
PRISM_ALLOW_CYCLE=false
PRISM_BATCH_SIZE=$batch_size
PRISM_REQUEST_PREFIX=s18
EOF

echo 'phase=config'
"${compose[@]}" config >/dev/null
echo 'phase=preflight'
python3 "$root/scripts/docker_production_preflight.py" --root "$root" --secret-dir "$secret_dir" --output "$evidence_dir/preflight.json"
echo 'phase=build'
"${compose[@]}" build server telemetry-client >/dev/null
echo 'phase=smoke'
"${compose[@]}" up -d server >/dev/null
smoke="$("${compose[@]}" run -T --rm --no-deps -e PRISM_FRAME_TARGET=3 telemetry-client)"
echo "$smoke" > "$evidence_dir/smoke.json"
echo "$smoke" | grep -q '"frames_ok": 3'
echo 'phase=100k'
set +e
result="$("${compose[@]}" run -T --rm --no-deps telemetry-client 2>&1)"
exit_code=$?
set -e
printf '%s\n' "$result" > "$evidence_dir/telemetry.log"
(( exit_code == 0 ))
echo "$result" | grep -q '"frames_sent": '$frames
echo "$result" | grep -q '"frames_ok": '$frames
echo "$result" | grep -q '"synthetic_cycle": false'
echo "$result" | grep -q '"dataset_sha256":'
echo "$result" | grep -q '"effective_user":'
echo 'phase=lifecycle'
"${compose[@]}" stop -t 10 server >/dev/null
"${compose[@]}" up -d server >/dev/null
echo 'phase=reconnect'
reconnect="$("${compose[@]}" run -T --rm --no-deps -e PRISM_FRAME_TARGET=3 telemetry-client)"
echo "$reconnect" > "$evidence_dir/reconnect.json"
echo "$reconnect" | grep -q '"frames_ok": 3'
echo 'phase=cleanup'
python3 - "$result" "$exit_code" "$evidence_dir/result.json" <<'PY'
import json, pathlib, sys
payload = json.loads(sys.argv[1].splitlines()[-1])
payload["exit_code"] = int(sys.argv[2])
pathlib.Path(sys.argv[3]).write_text(json.dumps(payload, sort_keys=True) + "\n", encoding="utf-8")
PY
cat "$evidence_dir/result.json"
