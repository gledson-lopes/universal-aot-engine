# syntax=docker/dockerfile:1
# ---------------------------------------------------------------------------
# STAGE 1: Compiler (Builder)
# ---------------------------------------------------------------------------
FROM rust:bookworm AS compiler

# System-wide Rust & Cargo paths
ENV RUSTUP_HOME=/usr/local/rustup \
    CARGO_HOME=/usr/local/cargo \
    PATH=/usr/local/cargo/bin:${PATH}

RUN apt-get update && apt-get install -y --no-install-recommends \
        build-essential \
        clang \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

COPY . .

# Build release binary using BuildKit cache mounts
RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/usr/local/cargo/git \
    --mount=type=cache,target=/app/target \
    cargo build --release \
    && cp /app/target/release/enum /tmp/universal-aot-engine

# ---------------------------------------------------------------------------
# STAGE 2: Development (devcontainers / CI)
# ---------------------------------------------------------------------------
FROM compiler AS dev

ENV RUSTUP_HOME=/usr/local/rustup \
    CARGO_HOME=/usr/local/cargo \
    PATH=/usr/local/cargo/bin:${PATH}

# Ensure non-login subshells (like Zed execs) always source Cargo's path
RUN echo 'export PATH="/usr/local/cargo/bin:${PATH}"' >> /etc/bash.bashrc \
    && echo 'export PATH="/usr/local/cargo/bin:${PATH}"' > /etc/profile.d/cargo.sh

WORKDIR /workspaces/universal-aot-engine

CMD ["sleep", "infinity"]

# ---------------------------------------------------------------------------
# STAGE 3: Runtime
# ---------------------------------------------------------------------------
FROM debian:bookworm-slim AS runtime

RUN apt-get update && apt-get install -y --no-install-recommends \
        ca-certificates \
    && rm -rf /var/lib/apt/lists/* \
    && useradd -m appuser

COPY --from=compiler /tmp/universal-aot-engine /usr/local/bin/universal-aot-engine

RUN mkdir -p /src && chown appuser:appuser /src

USER appuser
WORKDIR /src

ENTRYPOINT ["/usr/local/bin/universal-aot-engine"]
