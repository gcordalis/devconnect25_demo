# Use the official Rust image to build the app
FROM rust:1.90 as builder

WORKDIR /workspace

# Copy the manifest and source
COPY . .

# Build for release
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