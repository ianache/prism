FROM rust:1.85-bookworm AS builder

WORKDIR /src
COPY Cargo.toml Cargo.lock ./
COPY crates ./crates
RUN cargo build -p prism-runner --release

FROM python:3.12-slim-bookworm AS runtime

RUN useradd --create-home --uid 10001 prism
WORKDIR /opt/prism
COPY --from=builder /src/target/release/prism-run /usr/local/bin/prism-run
COPY docker/entrypoint.sh /usr/local/bin/prism-entrypoint.sh
COPY docker/healthcheck.py docker/client.py docker/telemetry_client.py /opt/prism/
RUN mkdir -p /run/prism /run/secrets \
    && chown -R prism:prism /opt/prism /run/prism /run/secrets \
    && chmod 0555 /usr/local/bin/prism-run /usr/local/bin/prism-entrypoint.sh

USER prism
ENTRYPOINT ["/bin/sh", "/usr/local/bin/prism-entrypoint.sh"]
