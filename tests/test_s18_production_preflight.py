from pathlib import Path


ROOT = Path(__file__).parents[1]


def test_preflight_script_exposes_sanitized_identity_contract():
    text = (ROOT / "scripts" / "docker_production_preflight.py").read_text(encoding="utf-8")
    assert "10001" in text
    assert "PRISM_ALLOW_CYCLE" in text
    assert "S18_PREFLIGHT_OK" in text
    assert "token" not in text.lower().split("print", 1)[-1]


def test_production_overlay_keeps_non_root_user():
    text = (ROOT / "docker-compose.production.yml").read_text(encoding="utf-8")
    assert 'user: "10001:10001"' in text
