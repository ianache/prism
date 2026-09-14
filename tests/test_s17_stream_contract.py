from pathlib import Path


ROOT = Path(__file__).parents[1]


def test_compose_supports_stream_input_and_production_profile():
    text = (ROOT / "docker-compose.yml").read_text(encoding="utf-8")
    assert "PRISM_INPUT_STREAM" in text
    assert "PRISM_CONTAINER_USER" in text
    production = (ROOT / "docker-compose.production.yml").read_text(encoding="utf-8")
    assert 'user: "10001:10001"' in production


def test_telemetry_client_has_sequential_stream_reader():
    text = (ROOT / "docker" / "telemetry_client.py").read_text(encoding="utf-8")
    assert "PRISM_INPUT_STREAM" in text
    assert "iter_stream_payloads" in text
    assert "synthetic_cycle" in text


def test_stream_generator_documents_real_distinct_corpus_contract():
    text = (ROOT / "scripts" / "generate_telemetry_stream.py").read_text(encoding="utf-8")
    assert "100_000" in text or "100000" in text
    assert "sha256" in text
    assert "crc32c" in text
