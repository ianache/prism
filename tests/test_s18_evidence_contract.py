from pathlib import Path


ROOT = Path(__file__).parents[1]


def test_s18_evidence_contains_release_gate_fields():
    text = (ROOT / "docs" / "evidence" / "rust-s18-production-readiness-2026-09-13.md").read_text(encoding="utf-8")
    for field in ("digest", "UID/GID", "lifecycle", "frames_ok", "duration", "sanitized"):
        assert field.lower() in text.lower()
