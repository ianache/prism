from pathlib import Path


ROOT = Path(__file__).parents[1]


def test_compose_declares_telemetry_profile_and_dataset_configuration():
    text = (ROOT / "docker-compose.yml").read_text(encoding="utf-8")
    assert "telemetry-client:" in text
    assert 'profiles: ["telemetry"]' in text
    assert "PRISM_INPUT_PATH" in text
    assert "PRISM_FRAME_TARGET" in text
    assert "PRISM_REQUEST_PREFIX" in text
    assert "/data/frames:ro" in text


def test_telemetry_client_exposes_deterministic_tls_ingestion_contract():
    text = (ROOT / "docker" / "telemetry_client.py").read_text(encoding="utf-8")
    assert "PRISM_INPUT_PATH" in text
    assert "PRISM_FRAME_TARGET" in text
    assert "PRISM_REQUEST_PREFIX" in text
    assert "payload_hex" in text
    assert "auth_token" in text
    assert "sha256" in text
    assert "endswith" in text
    assert "S15_TELEMETRY_OK" in text
    assert "PRISM_ALLOW_CYCLE" in text
    assert "PRISM_BATCH_SIZE" in text


def test_documented_telemetry_flow_has_smoke_and_100k_commands():
    guide = (ROOT / "docs" / "guia-servidor-supervisor-cliente-tls.md").read_text(encoding="utf-8")
    assert "--profile telemetry" in guide
    assert 'PRISM_FRAME_TARGET = "100000"' in guide
    assert "docker_telemetry_smoke.ps1" in guide
    assert (ROOT / "scripts" / "docker_telemetry_smoke.ps1").exists()
