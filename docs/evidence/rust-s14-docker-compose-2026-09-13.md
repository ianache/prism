# Evidencia S14: Docker Compose operational stack

Fecha: 2026-09-13

## Implementación

- `Dockerfile` multi-stage compila `prism-run` en release y usa runtime Python slim.
- `docker-compose.yml` define `server` y `client` bajo el perfil `client`.
- TLS, clave y token se montan desde el host como `:ro`.
- `docker/entrypoint.sh` activa el sentinel ante `TERM`/`INT` para permitir `DRAINING`.
- `docker/healthcheck.py` valida TLS + token + respuesta B2.
- `docker/client.py` ejecuta una solicitud JSONL B2 autenticada.
- `.dockerignore` excluye secretos, `target`, corpus, documentación y `graphify-out`.

## Verificaciones

| Comando | Resultado |
|---|---|
| contrato estructural S14 mediante Python | `S14_CONTRACT_OK` |
| `docker compose --env-file .env.example config` | PASS; configuración del servidor resuelta |
| `docker compose --env-file .env.example --profile client config` | PASS; servidor y cliente resueltos |
| `git diff --check` | PASS |
| `python -m compileall -q docker` | PASS |
| `cargo test --workspace --release` | PASS; código 0 |
| `docker compose build` | NO EJECUTADO; el daemon Docker no respondió en `dockerDesktopLinuxEngine` |
| smoke TLS S14 | PENDIENTE; requiere Docker Desktop activo y certificados PEM reales |

La suite Rust incluyó TLS E2E S12: 3/3 pruebas correctas. La meta de capacidad
de 100.000 frames no fue modificada.

## Limitaciones

No se reclama validación runtime de la imagen ni del ciclo `up`/`client`/`down`
hasta que Docker Desktop esté iniciado. La configuración está preparada para
usar certificados PEM y un token real mediante `.env`, sin incluir secretos en
la imagen ni en Git.
