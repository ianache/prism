from pathlib import Path


ROOT = Path(__file__).parents[1]


def test_production_readiness_can_execute_the_http_smoke_mode():
    workflow = (ROOT / ".github" / "workflows" / "production-readiness.yml").read_text(encoding="utf-8")
    runner = (ROOT / "scripts" / "docker_production_readiness.sh").read_text(encoding="utf-8")
    assert "PRISM_PROTOCOL: http" in workflow
    assert "--frames 3" in workflow
    assert "tests/test_s21_http_compose_contract.py" in workflow
    assert "tests/test_s21_http_contract.py" in workflow
    assert "PRISM_PROTOCOL=${PRISM_PROTOCOL:-tcp}" in runner


def test_http_manual_flow_is_documented_and_sanitized():
    guide = (ROOT / "docs" / "guia-servidor-supervisor-cliente-tls.md").read_text(encoding="utf-8")
    assert "curl" in guide
    assert "/healthz" in guide
    assert "/readyz" in guide
    assert "Authorization: Bearer" in guide
    assert "100.000" in guide or "100000" in guide
