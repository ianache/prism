import json

from scripts.build_evidence_bundle import build_bundle
from tools.evidence_bundle import validate_bundle


def test_builder_emits_deterministic_bundle_without_payloads(tmp_path):
    (tmp_path / "tcp-smoke.json").write_text(
        json.dumps({"name": "tcp smoke", "protocol": "tcp", "status": "PASS", "frames_sent": 3, "frames_ok": 3, "synthetic_cycle": False, "payload_hex": "secret-payload"}),
        encoding="utf-8",
    )
    output = tmp_path / "bundle.json"
    first = build_bundle(tmp_path, output, "wsl2", "a" * 40, "b" * 64)
    second = build_bundle(tmp_path, output, "wsl2", "a" * 40, "b" * 64)
    assert first == second
    assert validate_bundle(first) == []
    assert "payload_hex" not in json.dumps(first)
    assert (output).read_text(encoding="utf-8") == json.dumps(first, indent=2, sort_keys=True) + "\n"


def test_builder_marks_missing_case_as_not_executed(tmp_path):
    output = tmp_path / "bundle.json"
    bundle = build_bundle(tmp_path, output, "ci", "a" * 40, "b" * 64)
    missing = [case for case in bundle["cases"] if case["status"] == "NO EJECUTADA"]
    assert len(missing) == 4
    assert all(case["reason"] for case in missing)
    assert validate_bundle(bundle) == []
