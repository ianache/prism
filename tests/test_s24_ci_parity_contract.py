from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]


def test_ci_builds_and_uploads_s24_bundle():
    text = (ROOT / ".github" / "workflows" / "production-readiness.yml").read_text(encoding="utf-8")
    assert "build_evidence_bundle.py" in text
    assert "--source ci" in text
    assert "s24-evidence-bundle" in text
    assert "evidence-bundle.json" in text


def test_ci_preserves_comparable_tcp_gate_and_http_as_additional_measurement():
    text = (ROOT / ".github" / "workflows" / "production-readiness.yml").read_text(encoding="utf-8")
    assert "--frames 100000 --batch-size 64" in text
    assert "PRISM_PROTOCOL: http" in text
    assert "s20-linux-production-gate" in text
    assert "s21-http-production-smoke" in text
