import json

import pytest

from scripts.audit_evidence_bundle import audit_bundle


def test_audit_reports_status_counts_and_comparable_target():
    result = audit_bundle("tests/fixtures/s24/evidence-bundle-ci.json")
    assert result == "ok pass=4 fail=0 not_executed=0 frames=100000"


def test_audit_accepts_environmental_not_executed_fixture():
    result = audit_bundle("tests/fixtures/s24/evidence-bundle-not-executed.json")
    assert result == "ok pass=0 fail=0 not_executed=4 frames=100000"


def test_audit_rejects_invalid_bundle(tmp_path):
    data = json.loads(open("tests/fixtures/s24/evidence-bundle-ci.json", encoding="utf-8").read())
    data["cases"][2]["frames_ok"] = 99999
    path = tmp_path / "invalid.json"
    path.write_text(json.dumps(data), encoding="utf-8")
    with pytest.raises(ValueError, match="100000"):
        audit_bundle(path)
