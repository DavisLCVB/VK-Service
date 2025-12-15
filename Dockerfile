# Build stage
FROM rust:latest as builder

WORKDIR /app

# Install dependencies for building
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

# Copy manifests and create dummy src for dependency caching
COPY Cargo.toml Cargo.lock ./
RUN mkdir src && echo "fn main() {}" > src/main.rs
RUN cargo build --release --locked
RUN rm -rf src

# Copy real source and rebuild (only app code, deps are cached)
COPY src ./src
RUN touch src/main.rs
RUN cargo build --release --locked

# Runtime stage with minimal debian for better compatibility
FROM debian:bookworm-slim

WORKDIR /app

# Install minimal runtime dependencies
RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates \
    libssl3 \
    && rm -rf /var/lib/apt/lists/* \
    && groupadd -r appuser && useradd -r -g appuser appuser \
    && chown -R appuser:appuser /app

# Copy binary only (no assets needed)
COPY --from=builder /app/target/release/vk-service /app/vk-service

# Switch to non-root user
USER appuser

# Expose the port that Cloud Run expects
EXPOSE 8080

# Run the binary
ENTRYPOINT ["/app/vk-service"]
