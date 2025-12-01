# Multi-stage Dockerfile for Council Of Dicks
# Stage 1: Build Rust backend
# Stage 2: Build frontend assets  
# Stage 3: Final runtime image

# ============================================
# Stage 1: Build Rust Backend
# ============================================
FROM rust:slim-bookworm AS rust-builder

# Install build dependencies
# Note: glib and webkit are needed for Tauri dependencies (even for server-only build)
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    libglib2.0-dev \
    libwebkit2gtk-4.1-dev \
    build-essential \
    curl \
    wget \
    file \
    libgtk-3-dev \
    librsvg2-dev \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /build

# Copy Rust project files
COPY src-tauri/Cargo.toml src-tauri/Cargo.lock ./
COPY src-tauri/tauri.conf.json ./tauri.conf.json
COPY src-tauri/build.rs ./build.rs
COPY src-tauri/src ./src
COPY src-tauri/icons ./icons

# Build release binary (HTTP server mode only)
RUN cargo build --release

# ============================================
# Stage 2: Build Frontend (Optional - Future)
# ============================================
FROM node:20-slim AS frontend-builder

WORKDIR /build

# Copy package files
COPY package.json pnpm-lock.yaml ./

# Install pnpm and dependencies
RUN npm install -g pnpm && pnpm install --frozen-lockfile

# Copy source files
COPY src ./src
COPY public ./public
COPY index.html vite.config.ts tsconfig.json ./

# Build frontend
RUN pnpm build

# ============================================
# Stage 3: Runtime Image
# ============================================
FROM debian:bookworm-slim

# Install runtime dependencies
# Note: GTK/WebKit runtime libs needed even for server-only mode (Tauri deps)
RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    libglib2.0-0 \
    libgtk-3-0 \
    libwebkit2gtk-4.1-0 \
    librsvg2-2 \
    && rm -rf /var/lib/apt/lists/*

# Create non-root user
RUN useradd -m -u 1000 council && \
    mkdir -p /app/data && \
    chown -R council:council /app

WORKDIR /app

# Copy binary from builder
COPY --from=rust-builder /build/target/release/app ./council-server

# Copy test pages for web UI
COPY test-web-mode.html test-websocket.html ./

# Copy frontend assets (when available)
# COPY --from=frontend-builder /build/dist ./dist

# Set ownership
RUN chown -R council:council /app

# Switch to non-root user
USER council

# Expose ports
# 8080: HTTP API + WebSocket
# 9001: MCP server (optional)
EXPOSE 8080 9001

# Environment variables
ENV RUST_LOG=info
ENV OLLAMA_URL=http://host.docker.internal:11434
ENV OLLAMA_MODEL=qwen2.5-coder:7b

# Health check
HEALTHCHECK --interval=30s --timeout=3s --start-period=5s --retries=3 \
    CMD test -f /app/council-server || exit 1

# Start server
CMD ["./council-server", "--server", "--host", "0.0.0.0"]
