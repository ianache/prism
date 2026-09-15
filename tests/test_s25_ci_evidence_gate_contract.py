from pathlib import Path


WORKFLOW = Path(__file__).resolve().parents[1] / ".github" / "workflows" / "production-readiness.yml"


def test_ci_validates_and_audits_bundle_before_upload():
    text = WORKFLOW.read_text(encoding="utf-8")
    assert "Build validated S24 evidence bundle" in text
    assert "Audit S24 evidence bundle" in text
    assert "scripts/audit_evidence_bundle.py" in text
    assert "name: s25-production-evidence" in text
    assert text.count("if: always()") >= 3


def test_ci_keeps_comparable_tcp_gate_and_http_additional_measurement():
    text = WORKFLOW.read_text(encoding="utf-8")
    assert "--frames 100000 --batch-size 64" in text
    assert "PRISM_PROTOCOL: http" in text
    assert "s20-linux-production-gate" in text
    assert "s21-http-production-smoke" in text
