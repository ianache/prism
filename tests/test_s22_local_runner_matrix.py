from pathlib import Path


ROOT = Path(__file__).parents[1]
EVIDENCE = ROOT / "docs" / "evidence" / "rust-s22-local-linux-production-gate-2026-09-15.md"


def test_local_matrix_has_all_protocol_and_capacity_cases():
    assert EVIDENCE.exists()
    text = EVIDENCE.read_text(encoding="utf-8")
    for case in ("tcp smoke", "http smoke", "tcp 100000", "http 100000"):
        assert case in text.lower()
    assert "100000" in text


def test_unavailable_local_environment_is_not_reported_as_pass():
    text = EVIDENCE.read_text(encoding="utf-8")
    assert "NO EJECUTADA" in text
    assert "docker-desktop" in text.lower()
    assert "FAIL" in text


def test_local_evidence_has_no_secret_material():
    text = EVIDENCE.read_text(encoding="utf-8").lower()
    for marker in ("begin private key", "s18-ephemeral-token", "prism_auth_token="):
        assert marker not in text
