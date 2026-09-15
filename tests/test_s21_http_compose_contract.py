from pathlib import Path


ROOT = Path(__file__).parents[1]


def test_compose_exposes_protocol_with_tcp_default():
    compose = (ROOT / "docker-compose.yml").read_text(encoding="utf-8")
    env = (ROOT / ".env.example").read_text(encoding="utf-8")
    assert "PRISM_PROTOCOL" in compose
    assert "PRISM_PROTOCOL:-tcp" in compose
    assert "PRISM_PROTOCOL" in env


def test_entrypoint_passes_protocol_to_runner():
    entrypoint = (ROOT / "docker" / "entrypoint.sh").read_text(encoding="utf-8")
    assert "--protocol" in entrypoint
    assert "PRISM_PROTOCOL" in entrypoint


def test_healthcheck_and_client_have_explicit_http_modes():
    healthcheck = (ROOT / "docker" / "healthcheck.py").read_text(encoding="utf-8")
    client = (ROOT / "docker" / "client.py").read_text(encoding="utf-8")
    telemetry_client = (ROOT / "docker" / "telemetry_client.py").read_text(encoding="utf-8")
    assert "PRISM_PROTOCOL" in healthcheck
    assert "/healthz" in healthcheck
    assert "PRISM_PROTOCOL" in client
    assert "/v1/process" in client
    assert "PRISM_PROTOCOL" in telemetry_client
    assert "urllib.request" in telemetry_client
