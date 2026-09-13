# Evidencia Rust S11 — Secure Transport and Authentication

Fecha: 2026-09-13. Alcance: `prism-run` TCP local, TLS opcional y token por
mensaje; la meta de benchmark de 100.000 frames no se modifica.

## Implementación

- `--tls-cert` y `--tls-key` requieren pareja y se validan antes de `READY`.
- `rustls 0.23` y `rustls-pemfile 2` están limitados a `prism-runner`.
- El handshake se ejecuta tras `accept` y antes del contador de capacidad.
- `auth_token` es opcional en el sobre; cuando se configura el archivo, la
  comparación ocurre antes del dispatch y los fallos son recuperables.
- No se registran certificados generados, claves ni tokens.

## Verificación

El paquete `prism-runner` pasa 21 pruebas Rust, incluyendo CLI seguro,
validación de configuración TLS, token vacío, redacción y autenticación en
banda. La suite Python existente de S7–S10 permanece como regresión.

Limitación de esta ejecución: el arnés de cliente TLS de proceso en Windows
abortó durante `complete_io` antes de JSONL por un cierre de socket; no se
presenta como handshake E2E exitoso. La carga/validación de PEM y el camino de
transporte compilan, pero queda pendiente una prueba de proceso TLS estable.

No se reclama P0, mTLS, rotación, despliegue remoto ni persistencia.
