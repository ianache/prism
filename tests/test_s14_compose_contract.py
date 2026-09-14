from pathlib import Path


ROOT = Path(__file__).parents[1]


def compose_text() -> str:
    return (ROOT / "docker-compose.yml").read_text(encoding="utf-8")


def test_compose_declares_server_and_profile_client_with_compose_secrets():
    text = compose_text()
    assert "server:" in text
    assert "client:" in text
    assert "profiles:" in text
    assert 'profiles: ["client"]' in text
    assert "PRISM_TLS_CERT" in text
    assert "PRISM_TLS_KEY" in text
    assert "PRISM_AUTH_TOKEN" in text
    assert "secrets:" in text
    assert "/run/secrets/" in text
    assert "healthcheck:" in text
    assert "stop_grace_period" in text


def test_dockerignore_excludes_secrets_and_build_artifacts():
    text = (ROOT / ".dockerignore").read_text(encoding="utf-8")
    assert "secrets" in text
    assert "target" in text
    assert "graphify-out" in text


def test_compose_passes_capacity_and_lifecycle_configuration_to_runner():
    text = compose_text()
    for variable in (
        "PRISM_BIND",
        "PRISM_ROUTE",
        "PRISM_WORKERS",
        "PRISM_CONNECTION_QUEUE",
        "PRISM_DRAIN_TIMEOUT_MS",
    ):
        assert variable in text
    entrypoint = (ROOT / "docker" / "entrypoint.sh").read_text(encoding="utf-8")
    for flag in ("--route", "--listen", "--workers", "--connection-queue", "--shutdown-file", "--drain-timeout-ms"):
        assert flag in entrypoint


def test_docker_lifecycle_assets_exist_and_handle_term():
    dockerfile = (ROOT / "Dockerfile").read_text(encoding="utf-8")
    entrypoint = (ROOT / "docker" / "entrypoint.sh").read_text(encoding="utf-8")
    assert "FROM" in dockerfile
    assert "prism-run" in dockerfile
    assert "trap on_term TERM INT" in entrypoint
    assert "shutdown.flag" in entrypoint
    assert (ROOT / "docker" / "healthcheck.py").exists()
    assert (ROOT / "docker" / "client.py").exists()


def test_env_example_has_safe_defaults_without_secret_values():
    text = (ROOT / ".env.example").read_text(encoding="utf-8")
    for variable in (
        "PRISM_BIND",
        "PRISM_ROUTE",
        "PRISM_WORKERS",
        "PRISM_CONNECTION_QUEUE",
        "PRISM_DRAIN_TIMEOUT_MS",
        "PRISM_HOST_PORT",
        "PRISM_TLS_CERT",
        "PRISM_TLS_KEY",
        "PRISM_AUTH_TOKEN",
    ):
        assert variable in text
    assert "super-secret" not in text.lower()


def test_guide_documents_compose_flow():
    text = (ROOT / "docs" / "guia-servidor-supervisor-cliente-tls.md").read_text(encoding="utf-8")
    assert "docker compose" in text
    assert "up -d --build" in text
    assert "down" in text
    assert "100.000" in text
