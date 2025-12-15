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

# Install ALL necessary runtime dependencies
RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates \
    libssl3 \
    libgcc-s1 \
    libc6 \
    && rm -rf /var/lib/apt/lists/*

# Create app user and set permissions
RUN groupadd -r appuser && useradd -r -g appuser appuser

# Copy binary with correct permissions
COPY --from=builder --chown=appuser:appuser /app/target/release/vk-service /app/vk-service
RUN chmod +x /app/vk-service

# Create a startup script for debugging
RUN echo '#!/bin/sh\n\
echo "=== CONTAINER STARTING ==="\n\
echo "User: $(whoami)"\n\
echo "Binary exists: $(ls -la /app/vk-service)"\n\
echo "Binary executable: $(test -x /app/vk-service && echo YES || echo NO)"\n\
echo "Starting application..."\n\
exec /app/vk-service' > /app/start.sh && \
    chmod +x /app/start.sh && \
    chown appuser:appuser /app/start.sh

# Switch to non-root user
USER appuser

# Expose the port that Cloud Run expects
EXPOSE 8080

# Use the startup script
ENTRYPOINT ["/app/start.sh"]
