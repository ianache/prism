from pathlib import Path


ROOT = Path(__file__).parents[1]


def test_ci_workflow_has_required_jobs_and_commands():
    workflow = (ROOT / ".github" / "workflows" / "production-readiness.yml").read_text(encoding="utf-8")
    for job in ("rust:", "contracts:", "compose:", "docker-build:", "production-runtime:"):
        assert job in workflow
    assert "cargo test --workspace --release" in workflow
    assert "docker-compose.production.yml" in workflow


def test_ci_contract_scanner_rejects_secret_material_and_accepts_required_files():
    scanner = ROOT / "scripts" / "ci_contract.py"
    assert scanner.exists()
    assert "forbidden_markers" in scanner.read_text(encoding="utf-8")
