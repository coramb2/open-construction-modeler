FROM ubuntu:24.04

ENV DEBIAN_FRONTEND=noninteractive
ENV RUSTUP_HOME=/usr/local/rustup
ENV CARGO_HOME=/usr/local/cargo
ENV PATH=/usr/local/cargo/bin:$PATH
ENV RUST_VERSION=1.93.1
ENV NODE_VERSION=20.20.0

# System dependencies for Tauri + WebKit
RUN apt-get update && apt-get install -y \
    curl \
    build-essential \
    pkg-config \
    libssl-dev \
    ca-certificates \
    libwebkit2gtk-4.1-dev \
    libgtk-3-dev \
    libayatana-appindicator3-dev \
    librsvg2-dev \
    libglib2.0-dev \
    libjavascriptcoregtk-4.1-dev \
    libsoup-3.0-dev \
    file \
    wget \
    && rm -rf /var/lib/apt/lists/*

# Rust
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs -o /tmp/rustup-init.sh \
    && echo "6c30b75a75b28a96fd913a037c8581b580080b6ee9b8169a3c0feb1af7fe8caf /tmp/rustup-init.sh" | sha256sum -c - \
    && sh /tmp/rustup-init.sh -y --no-modify-path --default-toolchain ${RUST_VERSION} \
    && rm /tmp/rustup-init.sh

# Node.js
RUN curl -fsSL https://deb.nodesource.com/setup_20.x -o /tmp/setup_node.sh \
    && echo "2c4c6683a17b6f4128898a7b521e3c8bb725a99ffaf1b5e32ac97c6fa7d381be /tmp/setup_node.sh" | sha256sum -c - \
    && bash /tmp/setup_node.sh \
    && rm /tmp/setup_node.sh \
    && apt-get install -y nodejs \
    && rm -rf /var/lib/apt/lists/*

# Tauri CLI
RUN cargo install tauri-cli --version "^2.0"

# Create a non-root user
RUN useradd -m -s /bin/bash appuser

WORKDIR /app
RUN chown -R appuser:appuser /app

RUN chown -R appuser:appuser /usr/local/rustup /usr/local/cargo

USER appuser