# Build stage
FROM rust:1.83-slim as builder

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
RUN cargo build --release --locked

# Runtime stage with distroless for minimal size and security
FROM gcr.io/distroless/cc-debian12:nonroot

WORKDIR /app

# Copy binary only (no assets needed)
COPY --from=builder /app/target/release/vk-service /app/vk-service

# Expose the port that Cloud Run expects
EXPOSE 8080

# Run as nonroot user (already set in distroless:nonroot)
ENTRYPOINT ["/app/vk-service"]
