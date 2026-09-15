import json
from pathlib import Path

import pytest

from scripts.build_evidence_bundle import build_bundle
from tools.evidence_bundle import validate_bundle


FIXTURES = Path(__file__).resolve().parent / "fixtures" / "s25"


def load(name):
    return json.loads((FIXTURES / name).read_text(encoding="utf-8"))


def test_valid_ci_fixture_passes_common_validator():
    assert validate_bundle(load("valid-bundle.json")) == []


def test_partial_gate_fixture_is_rejected():
    errors = validate_bundle(load("invalid-partial-gate.json"))
    assert any("100000" in error for error in errors)


def test_secret_fixture_is_rejected_and_not_executed_is_not_acceptance():
    errors = validate_bundle(load("invalid-secret.json"))
    assert any("secret" in error.lower() for error in errors)
    not_executed = load("valid-bundle.json")
    not_executed["cases"][2] = {"name": "tcp 100000", "protocol": "tcp", "status": "NO EJECUTADA", "reason": "runner unavailable"}
    assert validate_bundle(not_executed) == []
    assert not_executed["cases"][2]["status"] != "PASS"


def test_builder_combines_tcp_and_http_evidence(tmp_path):
    tcp = tmp_path / "tcp"
    http = tmp_path / "http"
    tcp.mkdir()
    http.mkdir()
    tcp.joinpath("smoke.json").write_text(json.dumps({"frames_sent": 3, "frames_ok": 3, "synthetic_cycle": False}), encoding="utf-8")
    tcp.joinpath("result.json").write_text(json.dumps({"frames_sent": 100000, "frames_ok": 100000, "synthetic_cycle": False}), encoding="utf-8")
    http.joinpath("smoke.json").write_text(json.dumps({"frames_sent": 3, "frames_ok": 3, "synthetic_cycle": False}), encoding="utf-8")
    output = tmp_path / "bundle.json"

    bundle = build_bundle(tcp, output, "ci", "abc", "unknown", http_input_dir=http)

    assert [(case["name"], case["status"]) for case in bundle["cases"]] == [
        ("tcp smoke", "PASS"),
        ("http smoke", "PASS"),
        ("tcp 100000", "PASS"),
        ("http 100000", "NO EJECUTADA"),
    ]
