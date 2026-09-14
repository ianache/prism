from pathlib import Path


ROOT = Path(__file__).parents[1]


def test_runtime_script_has_explicit_acceptance_phases_and_defaults():
    text = (ROOT / "scripts" / "docker_production_readiness.sh").read_text(encoding="utf-8")
    for phase in ("smoke", "100k", "lifecycle", "reconnect", "cleanup"):
        assert phase in text
    assert "100000" in text
    assert "PRISM_ALLOW_CYCLE=false" in text
    assert "dataset_sha256" in text
    assert "effective_user" in text


def test_runtime_script_has_cleanup_trap_and_sanitized_evidence():
    text = (ROOT / "scripts" / "docker_production_readiness.sh").read_text(encoding="utf-8")
    assert "trap cleanup EXIT" in text
    assert "private key" not in text.lower()
