# syntax=docker/dockerfile:1.7

# ── Stage 1: Build ────────────────────────────────────────────
FROM rust:1.93-slim@sha256:7e6fa79cf81be23fd45d857f75f583d80cfdbb11c91fa06180fd747fda37a61d AS builder

WORKDIR /app
ARG OCTOCLAW_CARGO_FEATURES=""

# Install build dependencies
RUN --mount=type=cache,target=/var/cache/apt,sharing=locked \
    --mount=type=cache,target=/var/lib/apt,sharing=locked \
    apt-get update && apt-get install -y \
        pkg-config \
    && rm -rf /var/lib/apt/lists/*

# 1. Copy manifests to cache dependencies
COPY Cargo.toml Cargo.lock ./
COPY build.rs build.rs
COPY crates/robot-kit/Cargo.toml crates/robot-kit/Cargo.toml
COPY crates/octoclaw-types/Cargo.toml crates/octoclaw-types/Cargo.toml
COPY crates/octoclaw-core/Cargo.toml crates/octoclaw-core/Cargo.toml
# Create dummy targets declared in Cargo.toml so manifest parsing succeeds.
RUN mkdir -p src benches crates/robot-kit/src crates/octoclaw-types/src crates/octoclaw-core/src \
    && echo "fn main() {}" > src/main.rs \
    && echo "fn main() {}" > benches/agent_benchmarks.rs \
    && echo "pub fn placeholder() {}" > crates/robot-kit/src/lib.rs \
    && echo "pub fn placeholder() {}" > crates/octoclaw-types/src/lib.rs \
    && echo "pub fn placeholder() {}" > crates/octoclaw-core/src/lib.rs
RUN --mount=type=cache,id=octoclaw-cargo-registry,target=/usr/local/cargo/registry,sharing=locked \
    --mount=type=cache,id=octoclaw-cargo-git,target=/usr/local/cargo/git,sharing=locked \
    --mount=type=cache,id=octoclaw-target,target=/app/target,sharing=locked \
    if [ -n "$OCTOCLAW_CARGO_FEATURES" ]; then \
      cargo build --release --features "$OCTOCLAW_CARGO_FEATURES"; \
    else \
      cargo build --release --locked; \
    fi
RUN rm -rf src benches crates/robot-kit/src crates/octoclaw-types/src crates/octoclaw-core/src

# 2. Copy only build-relevant source paths (avoid cache-busting on docs/tests/scripts)
COPY src/ src/
COPY benches/ benches/
COPY crates/ crates/
COPY firmware/ firmware/
COPY templates/ templates/
COPY web/ web/
# Keep release builds resilient when frontend dist assets are not prebuilt in Git.
RUN mkdir -p web/dist && \
    if [ ! -f web/dist/index.html ]; then \
      printf '%s\n' \
        '<!doctype html>' \
        '<html lang="en">' \
        '  <head>' \
        '    <meta charset="utf-8" />' \
        '    <meta name="viewport" content="width=device-width,initial-scale=1" />' \
        '    <title>OctoClaw Dashboard</title>' \
        '  </head>' \
        '  <body>' \
        '    <h1>OctoClaw Dashboard Unavailable</h1>' \
        '    <p>Frontend assets are not bundled in this build. Build the web UI to populate <code>web/dist</code>.</p>' \
        '  </body>' \
        '</html>' > web/dist/index.html; \
    fi
RUN --mount=type=cache,id=octoclaw-cargo-registry,target=/usr/local/cargo/registry,sharing=locked \
    --mount=type=cache,id=octoclaw-cargo-git,target=/usr/local/cargo/git,sharing=locked \
    --mount=type=cache,id=octoclaw-target,target=/app/target,sharing=locked \
    if [ -n "$OCTOCLAW_CARGO_FEATURES" ]; then \
      cargo build --release --features "$OCTOCLAW_CARGO_FEATURES"; \
    else \
      cargo build --release --locked; \
    fi && \
    cp target/release/octoclaw /app/octoclaw && \
    strip /app/octoclaw

# Prepare runtime directory structure and default config inline (no extra stage)
RUN mkdir -p /octoclaw-data/.octoclaw /octoclaw-data/workspace && \
    cat > /octoclaw-data/.octoclaw/config.toml <<EOF && \
    chown -R 65534:65534 /octoclaw-data
workspace_dir = "/octoclaw-data/workspace"
config_path = "/octoclaw-data/.octoclaw/config.toml"
api_key = ""
default_provider = "openrouter"
default_model = "anthropic/claude-sonnet-4-20250514"
default_temperature = 0.7

[gateway]
port = 42617
host = "127.0.0.1"
allow_public_bind = false
EOF

# ── Stage 2: Development Runtime (Debian) ────────────────────
FROM debian:trixie-slim@sha256:1d3c811171a08a5adaa4a163fbafd96b61b87aa871bbc7aa15431ac275d3d430 AS dev

# Install essential runtime dependencies only (use docker-compose.override.yml for dev tools)
RUN apt-get update && apt-get install -y \
    ca-certificates \
    curl \
    && rm -rf /var/lib/apt/lists/*

COPY --from=builder /octoclaw-data /octoclaw-data
COPY --from=builder /app/octoclaw /usr/local/bin/octoclaw

# Overwrite minimal config with DEV template (Ollama defaults)
COPY dev/config.template.toml /octoclaw-data/.octoclaw/config.toml
RUN chown 65534:65534 /octoclaw-data/.octoclaw/config.toml

# Environment setup
# Use consistent workspace path
ENV OCTOCLAW_WORKSPACE=/octoclaw-data/workspace
ENV HOME=/octoclaw-data
# Defaults for local dev (Ollama) - matches config.template.toml
ENV PROVIDER="ollama"
ENV OCTOCLAW_MODEL="llama3.2"
ENV OCTOCLAW_GATEWAY_PORT=42617

# Note: API_KEY is intentionally NOT set here to avoid confusion.
# It is set in config.toml as the Ollama URL.

WORKDIR /octoclaw-data
USER 65534:65534
EXPOSE 42617
ENTRYPOINT ["octoclaw"]
CMD ["gateway"]

# ── Stage 3: Production Runtime (Distroless) ─────────────────
FROM gcr.io/distroless/cc-debian13:nonroot@sha256:84fcd3c223b144b0cb6edc5ecc75641819842a9679a3a58fd6294bec47532bf7 AS release

COPY --from=builder /app/octoclaw /usr/local/bin/octoclaw
COPY --from=builder /octoclaw-data /octoclaw-data

# Environment setup
ENV OCTOCLAW_WORKSPACE=/octoclaw-data/workspace
ENV HOME=/octoclaw-data
# Default provider and model are set in config.toml, not here,
# so config file edits are not silently overridden
#ENV PROVIDER=
ENV OCTOCLAW_GATEWAY_PORT=42617

# API_KEY must be provided at runtime!

WORKDIR /octoclaw-data
USER 65534:65534
EXPOSE 42617
ENTRYPOINT ["octoclaw"]
CMD ["gateway"]
