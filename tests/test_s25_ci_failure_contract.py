from pathlib import Path


WORKFLOW = Path(__file__).resolve().parents[1] / ".github" / "workflows" / "production-readiness.yml"


def test_audit_step_is_fail_fast_but_artifact_upload_is_always():
    text = WORKFLOW.read_text(encoding="utf-8")
    audit_start = text.index("- name: Audit S24 evidence bundle")
    upload_start = text.index("- name: Upload S25 production evidence")
    audit = text[audit_start:upload_start]
    upload = text[upload_start:]
    assert "if: always()" in audit
    assert "run: python scripts/audit_evidence_bundle.py" in audit
    assert "continue-on-error" not in audit
    assert "if: always()" in upload


def test_ci_failure_documentation_distinguishes_fail_and_not_executed():
    guide = (Path(__file__).resolve().parents[1] / "docs" / "guia-servidor-supervisor-cliente-tls.md").read_text(encoding="utf-8")
    assert "NO EJECUTADA" in guide
    assert "artifact" in guide.lower()
