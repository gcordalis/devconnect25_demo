# Use the official Rust image to build the app
FROM rust:1.90 as builder

WORKDIR /workspace

# Copy only the manifest files first (for better caching)
COPY Cargo.toml Cargo.lock ./

# Create a dummy main.rs to build dependencies
RUN mkdir src && echo "fn main() {}" > src/main.rs
RUN cargo build --release
RUN rm src/main.rs

# Now copy the actual source code
COPY src/ ./src/

# Build the final binary
RUN cargo build --release

# Use a minimal base image for the runtime
FROM debian:bookworm-slim

WORKDIR /app

# Copy the built binary from the builder
COPY --from=builder /workspace/target/release/swissbank /app/swissbank

# Expose the port (adjust if needed)
EXPOSE 3000

# Run the server
CMD ["/app/swissbank"]