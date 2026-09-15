import json
from pathlib import Path

import pytest

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
