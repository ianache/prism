from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]


def test_production_gate_keeps_fixed_100k_and_required_phases():
    text = (ROOT / "scripts" / "docker_production_readiness.sh").read_text(encoding="utf-8")
    assert "frames=100000" in text
    assert "batch_size=64" in text
    for phase in (
        "phase=config",
        "phase=preflight",
        "phase=smoke",
        "phase=100k",
        "phase=lifecycle",
        "phase=reconnect",
        "phase=cleanup",
    ):
        assert phase in text
    assert '"frames_sent": ' in text
    assert '"frames_ok": ' in text
    assert '"synthetic_cycle": false' in text


def test_production_gate_uses_non_root_and_disables_synthetic_cycle():
    text = (ROOT / "scripts" / "docker_production_readiness.sh").read_text(encoding="utf-8")
    assert "PRISM_CONTAINER_USER=10001:10001" in text
    assert "PRISM_ALLOW_CYCLE=false" in text


def test_production_gate_exports_environment_secrets_for_non_root_compose_mounts():
    runner = (ROOT / "scripts" / "docker_production_readiness.sh").read_text(encoding="utf-8")
    overlay = (ROOT / "docker-compose.production.yml").read_text(encoding="utf-8")
    for name in ("PRISM_TLS_CERT_CONTENT", "PRISM_TLS_KEY_CONTENT", "PRISM_AUTH_TOKEN_CONTENT"):
        assert f'export {name}=' in runner
        assert f"environment: {name}" in overlay
    assert 'uid: "10001"' in overlay
    assert 'gid: "10001"' in overlay
    assert "mode: 0400" in overlay
