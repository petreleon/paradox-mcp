FROM rust:slim-bookworm AS builder

# Install build dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    clang \
    libclang-dev \
    pxlib-dev \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app
COPY . .

# Build the Rust application
RUN cargo build --release

# --- Runtime Image ---
FROM debian:bookworm-slim

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    pxlib1 \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app
# Copy the binary
COPY --from=builder /app/target/release/paradox-mcp /usr/local/bin/paradox-mcp

# The container will run the generated binary.
ENTRYPOINT ["paradox-mcp"]
CMD ["--help"]
