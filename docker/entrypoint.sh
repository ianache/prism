#!/bin/sh
set -eu

sentinel="${PRISM_SHUTDOWN_FILE:-/run/prism/shutdown.flag}"
term_requested=0
mkdir -p "$(dirname "$sentinel")"
rm -f "$sentinel"

on_term() {
    term_requested=1
    touch "$sentinel"
}

trap on_term TERM INT

/usr/local/bin/prism-run \
    --route "${PRISM_ROUTE:-b2}" \
    --protocol "${PRISM_PROTOCOL:-tcp}" \
    --listen "${PRISM_BIND:-0.0.0.0:9000}" \
    --workers "${PRISM_WORKERS:-2}" \
    --connection-queue "${PRISM_CONNECTION_QUEUE:-2}" \
    --tls-cert "${PRISM_TLS_CERT_FILE:-/run/secrets/server-cert.pem}" \
    --tls-key "${PRISM_TLS_KEY_FILE:-/run/secrets/server-key.pem}" \
    --auth-token-file "${PRISM_AUTH_TOKEN_FILE:-/run/secrets/auth-token.txt}" \
    --shutdown-file "$sentinel" \
    --drain-timeout-ms "${PRISM_DRAIN_TIMEOUT_MS:-5000}" &
runner_pid=$!

set +e
wait "$runner_pid"
runner_status=$?
if [ "$term_requested" -eq 1 ]; then
    wait "$runner_pid"
    runner_status=$?
fi
exit "$runner_status"
