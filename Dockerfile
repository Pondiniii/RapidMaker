# syntax=docker/dockerfile:1.7

FROM rust:1.76-slim AS builder
WORKDIR /app

# Install build dependencies
RUN apt-get update && apt-get install -y --no-install-recommends \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

# Cache dependencies
COPY Cargo.toml ./
COPY crates/price-engine/Cargo.toml crates/price-engine/Cargo.toml
COPY crates/rapidmaker-api/Cargo.toml crates/rapidmaker-api/Cargo.toml
RUN cargo fetch

# Build sources
COPY . .
RUN mkdir -p frontend/dist && cargo build --release -p rapidmaker-api

FROM debian:bookworm-slim AS runtime
WORKDIR /app

RUN apt-get update && apt-get install -y --no-install-recommends \
        ca-certificates \
    && rm -rf /var/lib/apt/lists/*

COPY --from=builder /app/target/release/rapidmaker-api /usr/local/bin/rapidmaker-api
COPY frontend/dist /app/frontend/dist

ENV RUST_LOG=info \
    RAPIDMAKER_BIND=0.0.0.0:8080 \
    RAPIDMAKER_UPLOAD_DIR=/tmp/rapidmaker

EXPOSE 8080

CMD ["rapidmaker-api"]
