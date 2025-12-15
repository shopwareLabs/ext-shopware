# Dockerfile for building ext-shopware PHP extension
# Supports multiple PHP versions and architectures (x86_64, arm64)
# Can be based on either Alpine (musl) or Debian (glibc)

ARG PHP_VERSION=8.4
ARG BASE_IMAGE=bookworm

# Rust builder stage - use official Rust image
# Using 1.92+ to support Rust edition 2024
FROM rust:1.92 AS rust-builder

# PHP builder stage with Rust from rust-builder
FROM php:${PHP_VERSION}-cli-${BASE_IMAGE} AS builder

ARG BASE_IMAGE=bookworm

# Copy Rust toolchain from rust-builder stage
COPY --from=rust-builder /usr/local/cargo /usr/local/cargo
COPY --from=rust-builder /usr/local/rustup /usr/local/rustup

ENV CARGO_HOME=/usr/local/cargo \
    RUSTUP_HOME=/usr/local/rustup \
    PATH=/usr/local/cargo/bin:$PATH

# Install system dependencies based on the base image
RUN if echo "$BASE_IMAGE" | grep -q "alpine"; then \
        apk add --no-cache \
            build-base \
            vips-dev \
            pkgconfig \
            git; \
    else \
        apt-get update && \
        apt-get install -y --no-install-recommends \
            build-essential \
            libvips-dev \
            pkg-config \
            git \
            ca-certificates && \
        rm -rf /var/lib/apt/lists/*; \
    fi

# Verify Rust installation
RUN cargo --version && rustc --version

# Set working directory
WORKDIR /build

# Copy source files
COPY Cargo.toml Cargo.lock ./
COPY src ./src

# Build the extension in release mode
RUN cargo build --release

# Runtime stage (optional, for testing)
FROM php:${PHP_VERSION}-cli-${BASE_IMAGE}

ARG BASE_IMAGE=bookworm

# Install runtime dependencies
RUN if echo "$BASE_IMAGE" | grep -q "alpine"; then \
        apk add --no-cache vips; \
    else \
        apt-get update && \
        apt-get install -y --no-install-recommends libvips && \
        rm -rf /var/lib/apt/lists/*; \
    fi

# Copy built extension from builder
COPY --from=builder /build/target/release/libext_shopware.so /usr/local/lib/php/extensions/libext_shopware.so

# Enable the extension
RUN echo "extension=/usr/local/lib/php/extensions/libext_shopware.so" > /usr/local/etc/php/conf.d/ext-shopware.ini

# Set working directory
WORKDIR /app

# Copy composer files for testing
COPY composer.json composer.lock ./
COPY tests ./tests
COPY phpunit.xml ./

# Install composer
RUN curl -sS https://getcomposer.org/installer | php -- --install-dir=/usr/local/bin --filename=composer

# Install PHP dependencies
RUN composer install --no-dev --optimize-autoloader

# Default command
CMD ["php", "-m"]
