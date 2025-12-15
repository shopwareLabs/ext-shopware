# Dockerfile for building ext-shopware PHP extension
# Supports multiple PHP versions and architectures (x86_64, arm64)
# Can be based on either Alpine (musl) or Debian (glibc)

ARG PHP_VERSION=8.4
ARG BASE_IMAGE=debian

# Use multi-stage build for efficiency
FROM php:${PHP_VERSION}-cli-${BASE_IMAGE} as builder

# Install system dependencies based on the base image
RUN if [ "$BASE_IMAGE" = "alpine" ]; then \
        apk add --no-cache \
            build-base \
            vips-dev \
            pkgconfig \
            curl \
            git; \
    else \
        apt-get update && \
        apt-get install -y --no-install-recommends \
            build-essential \
            libvips-dev \
            pkg-config \
            curl \
            git \
            ca-certificates && \
        rm -rf /var/lib/apt/lists/*; \
    fi

# Install Rust
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y --default-toolchain stable
ENV PATH="/root/.cargo/bin:${PATH}"

# Set working directory
WORKDIR /build

# Copy source files
COPY Cargo.toml Cargo.lock ./
COPY src ./src

# Build the extension in release mode
RUN cargo build --release

# Runtime stage (optional, for testing)
FROM php:${PHP_VERSION}-cli-${BASE_IMAGE}

ARG BASE_IMAGE=debian

# Install runtime dependencies
RUN if [ "$BASE_IMAGE" = "alpine" ]; then \
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
