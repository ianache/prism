from tools.evidence_bundle import validate_bundle


def valid_bundle():
    return {
        "schema_version": "s24.v1",
        "source": "ci",
        "commit": "a" * 40,
        "digest": "b" * 64,
        "cases": [
            {"name": "tcp smoke", "protocol": "tcp", "status": "PASS", "frames_sent": 3, "frames_ok": 3, "synthetic_cycle": False},
            {"name": "http smoke", "protocol": "http", "status": "PASS", "frames_sent": 3, "frames_ok": 3, "synthetic_cycle": False},
            {"name": "tcp 100000", "protocol": "tcp", "status": "PASS", "frames_sent": 100000, "frames_ok": 100000},
            {"name": "http 100000", "protocol": "http", "status": "PASS", "frames_sent": 100000, "frames_ok": 100000},
        ],
    }


def test_validates_complete_bundle_and_comparable_tcp_gate():
    assert validate_bundle(valid_bundle()) == []


def test_requires_reason_for_not_executed_and_diagnostic_for_failure():
    bundle = valid_bundle()
    bundle["cases"][0] = {"name": "tcp smoke", "protocol": "tcp", "status": "NO EJECUTADA"}
    bundle["cases"][1] = {"name": "http smoke", "protocol": "http", "status": "FAIL"}
    errors = validate_bundle(bundle)
    assert any("reason" in error for error in errors)
    assert any("diagnostic" in error for error in errors)


def test_rejects_incomplete_comparable_gate_and_sensitive_values():
    bundle = valid_bundle()
    bundle["cases"][2]["frames_ok"] = 99999
    bundle["secret"] = "-----BEGIN " + "PRIVATE KEY-----"
    errors = validate_bundle(bundle)
    assert any("100000" in error for error in errors)
    assert any("secret" in error.lower() for error in errors)
