# Stage 1: Build the Rust binary
FROM rust:slim-bookworm AS builder

WORKDIR /usr/src/app

# Install build dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

# Copy manifests and pre-build dependencies to cache them
COPY Cargo.toml Cargo.lock ./
RUN mkdir src && echo "fn main() {}" > src/main.rs
RUN cargo build --release
RUN rm -f target/release/deps/gemini_audio_mcp*

# Copy actual source and build
COPY src ./src
RUN cargo build --release

# Stage 2: Final runtime image
FROM debian:bookworm-slim

WORKDIR /app

# Add OCI labels
LABEL org.opencontainers.image.title="Gemini Audio MCP"
LABEL org.opencontainers.image.source="https://github.com/jxoesneon/gemini-audio-mcp"
LABEL org.opencontainers.image.description="High-performance MCP server for Gemini 2.0/3.0 audio and Lyria 3 music generation"
LABEL org.opencontainers.image.authors="jxoesneon"
LABEL org.opencontainers.image.licenses="MIT"
LABEL org.opencontainers.image.url="https://github.com/jxoesneon/gemini-audio-mcp"
LABEL org.opencontainers.image.vendor="jxoesneon"
LABEL io.modelcontextprotocol.server.name="io.github.jxoesneon/gemini-audio-mcp"

# Install runtime dependencies: FFmpeg and SSL certificates
RUN apt-get update && apt-get install -y \
    ffmpeg \
    ca-certificates \
    libssl3 \
    && rm -rf /var/lib/apt/lists/*

# Copy binary from builder
COPY --from=builder /usr/src/app/target/release/gemini-audio-mcp /usr/local/bin/gemini-audio-mcp

# Environment variables for configuration
ENV GEMINI_API_KEY=""
ENV RUST_LOG="info"

# Create a volume for persistent data (audio outputs and config)
VOLUME ["/root/.local/share/gemini-audio-mcp"]

# MCP servers run over stdio
ENTRYPOINT ["gemini-audio-mcp"]
