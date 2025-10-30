# syntax=docker/dockerfile:1

# ===== Builder stage =====
FROM rust:1.84-slim AS builder

WORKDIR /app

# Install system deps for mongodb/openssl
RUN apt-get update \
    && apt-get install -y --no-install-recommends \
       build-essential clang pkg-config libssl-dev ca-certificates \
    && rm -rf /var/lib/apt/lists/*

# Copy manifest and source
COPY Cargo.toml Cargo.lock ./
COPY src ./src

# Build release binary (with cargo/target cache)
RUN --mount=type=cache,target=/usr/local/cargo/registry \
    cargo build --release && \
    cp target/release/mock-services /app/mock-services

# ===== Runtime stage =====
FROM debian:bookworm-slim AS runtime

RUN apt-get update \
    && apt-get install -y --no-install-recommends ca-certificates openssl \
    && rm -rf /var/lib/apt/lists/* \
    && useradd -m -u 10001 appuser

WORKDIR /app

COPY --from=builder /app/mock-services /app/mock-services
RUN mkdir -p /app/uploads

ENV PORT=3000
EXPOSE 3000

USER appuser

CMD ["/app/mock-services"]


