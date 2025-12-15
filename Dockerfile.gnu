# ARG for PHP version, default to 8.3
ARG PHP_VERSION=8.3
ARG DEBIAN_RELEASE=bullseye

FROM php:${PHP_VERSION}-cli-${DEBIAN_RELEASE} AS builder

# Install build dependencies
# clang/llvm is required for bindgen
# libvips-dev is required for the extension
RUN apt-get update && apt-get install -y \
    build-essential \
    curl \
    git \
    clang \
    libclang-dev \
    libvips-dev \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

# Install Rust
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
ENV PATH="/root/.cargo/bin:${PATH}"

WORKDIR /app

# Copy Cargo files first to cache dependencies
COPY Cargo.toml Cargo.lock ./

# Create a dummy main.rs to build dependencies and cache them
# This is a common trick to speed up builds
RUN mkdir src && echo "fn main() {}" > src/lib.rs && \
    cargo build --release && \
    rm -rf src target/release/deps/ext_shopware*

# Copy the actual source code
COPY . .

# Build the extension
# We need to touch the source file to force rebuild since we modified it above
RUN touch src/lib.rs && cargo build --release

# Rename dylib to so if needed (mostly for convenience)
RUN cp target/release/libext_shopware.so target/release/ext_shopware.so || true
RUN cp target/release/libext_shopware.dylib target/release/ext_shopware.so || true

# Verify it loads
RUN php -dextension=target/release/libext_shopware.so -m | grep ext-shopware

# Output stage
FROM scratch AS export
COPY --from=builder /app/target/release/libext_shopware.so /
