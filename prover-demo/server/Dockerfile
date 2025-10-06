# Use the official Rust image as the base
FROM rust:1.90 as builder

# Set working directory
WORKDIR /app

# Copy the Cargo.toml and Cargo.lock files
COPY Cargo.toml Cargo.lock ./

# Create a dummy main.rs to cache dependencies
RUN mkdir src && echo "fn main() {}" > src/main.rs

# Build dependencies (this layer will be cached if Cargo.toml doesn't change)
RUN cargo build --release && rm -rf src

# Copy the actual source code
COPY src ./src

# Build the application
RUN cargo build --release

# Runtime stage
FROM debian:bookworm-slim

# Install required system dependencies including OpenSSL
RUN apt-get update && apt-get install -y \
    ca-certificates \
    openssl \
    && rm -rf /var/lib/apt/lists/*

# Create certificate directory
RUN mkdir -p /app/certs

# Generate self-signed certificate for HTTPS WebSocket
RUN openssl req -x509 -nodes -days 365 -newkey rsa:2048 \
    -keyout /app/certs/server.key \
    -out /app/certs/server.crt \
    -subj "/C=US/ST=State/L=City/O=Organization/CN=localhost"

# Create a non-root user
RUN useradd -r -s /bin/false appuser

# Set working directory
WORKDIR /app

# Copy the binary from builder stage
COPY --from=builder /app/target/release/server /app/server

# Change ownership to appuser (but keep certs readable)
RUN chown -R appuser:appuser /app
RUN chmod 644 /app/certs/server.crt
RUN chmod 600 /app/certs/server.key
RUN chown appuser:appuser /app/certs/server.key /app/certs/server.crt

# Switch to non-root user
USER appuser

# Expose the ports (9816 for HTTP, 55688 for wstcp proxy)
EXPOSE 9816 55688

# Run the binary
CMD ["./server"]