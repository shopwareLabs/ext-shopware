# Building ext-shopware

This document describes how to build the ext-shopware PHP extension for various architectures and PHP versions.

## Overview

The extension can be built for:
- **Architectures**: x86_64 (amd64), arm64
- **PHP Versions**: 8.1, 8.2, 8.3, 8.4
- **C Library**: glibc (Debian/Ubuntu), musl (Alpine)

## Prerequisites

- Docker with buildx support
- For multi-architecture builds: QEMU (handled automatically by Docker Desktop or setup via docker/setup-qemu-action)

## Local Building

### Quick Start

Use the provided build script:

```bash
# Build for PHP 8.4 on Debian (x86_64)
./build-local.sh 8.4 bookworm x86_64

# Build for PHP 8.3 on Alpine (arm64)
./build-local.sh 8.3 alpine3.20 arm64

# Build for PHP 8.2 on Debian (arm64)
./build-local.sh 8.2 bookworm arm64
```

### Manual Docker Build

Build for specific configuration:

```bash
# Build for PHP 8.4 on Debian (glibc)
docker buildx build \
  --platform linux/amd64 \
  --build-arg PHP_VERSION=8.4 \
  --build-arg BASE_IMAGE=bookworm \
  --target builder \
  --output type=local,dest=./output \
  .

# Build for PHP 8.3 on Alpine (musl)
docker buildx build \
  --platform linux/arm64 \
  --build-arg PHP_VERSION=8.3 \
  --build-arg BASE_IMAGE=alpine3.20 \
  --target builder \
  --output type=local,dest=./output \
  .
```

The built extension will be in `./output/build/target/release/libext_shopware.so`

### Build Full Docker Image

Build a complete image with the extension and runtime dependencies:

```bash
# Build for PHP 8.4 on Debian
docker buildx build \
  --platform linux/amd64 \
  --build-arg PHP_VERSION=8.4 \
  --build-arg BASE_IMAGE=bookworm \
  -t ext-shopware:php8.4-debian \
  .

# Run tests in the container
docker run --rm ext-shopware:php8.4-debian php -m | grep -i shopware
```

### Testing the Extension

After building, test that the extension loads:

```bash
# Quick test
docker run --rm -v $(pwd)/artifacts:/ext \
  php:8.4-cli-bookworm \
  sh -c 'php -dextension=/ext/libext_shopware-php8.4-x86_64-glibc.so -m | grep -i shopware'
```

## Base Image Options

### Debian-based (glibc)
- `bookworm` - Debian 12 (recommended for glibc builds)
- `bullseye` - Debian 11

### Alpine-based (musl)
- `alpine3.20` - Alpine 3.20 (recommended for musl builds)
- `alpine3.19` - Alpine 3.19
- `alpine` - Latest Alpine

## GitHub Actions

The repository includes automated builds via GitHub Actions that:
1. Build the extension for all combinations of PHP versions, architectures, and libc types
2. Upload build artifacts
3. Test that extensions load correctly

### Workflow File

`.github/workflows/build-multi-arch.yml` - Multi-architecture build workflow

### Artifacts

Each successful build uploads an artifact named:
```
libext_shopware-php{VERSION}-{ARCH}-{LIBC}.so
```

Examples:
- `libext_shopware-php8.4-x86_64-glibc.so`
- `libext_shopware-php8.3-arm64-musl.so`

## Architecture Matrix

The build matrix includes:

| PHP Version | Architecture | Libc  | Base Image    |
|-------------|--------------|-------|---------------|
| 8.1         | x86_64       | glibc | bookworm      |
| 8.1         | x86_64       | musl  | alpine3.20    |
| 8.1         | arm64        | glibc | bookworm      |
| 8.1         | arm64        | musl  | alpine3.20    |
| 8.2         | x86_64       | glibc | bookworm      |
| 8.2         | x86_64       | musl  | alpine3.20    |
| 8.2         | arm64        | glibc | bookworm      |
| 8.2         | arm64        | musl  | alpine3.20    |
| 8.3         | x86_64       | glibc | bookworm      |
| 8.3         | x86_64       | musl  | alpine3.20    |
| 8.3         | arm64        | glibc | bookworm      |
| 8.3         | arm64        | musl  | alpine3.20    |
| 8.4         | x86_64       | glibc | bookworm      |
| 8.4         | x86_64       | musl  | alpine3.20    |
| 8.4         | arm64        | glibc | bookworm      |
| 8.4         | arm64        | musl  | alpine3.20    |

Total: 16 build combinations

## Troubleshooting

### Docker buildx not available

If you get an error about buildx:

```bash
docker buildx version
```

If not available, install it following the [official documentation](https://github.com/docker/buildx#installing).

### QEMU for cross-compilation

For building arm64 on x86_64 (or vice versa):

```bash
# Setup QEMU
docker run --rm --privileged multiarch/qemu-user-static --reset -p yes

# Verify
docker buildx ls
```

### libvips not found

Ensure the base image includes libvips development files. Both Debian and Alpine variants in the Dockerfile install the necessary dependencies.

### SSL certificate errors in Docker

If you encounter SSL certificate errors when building in Docker (e.g., in corporate environments with SSL inspection):

```
SSL certificate problem: self-signed certificate in certificate chain
```

This is typically caused by corporate SSL interceptors or security scanners. The Dockerfile uses multi-stage builds with the official Rust image which should work in most environments. If you still encounter issues:

1. Ensure your Docker daemon has access to the internet without SSL inspection
2. Use GitHub Actions instead - the automated builds will work correctly there
3. Configure your corporate proxy/CA certificates in the Docker build if needed

## Development

For local development without Docker:

```bash
# Install system dependencies (Ubuntu/Debian)
sudo apt-get install libvips-dev pkg-config

# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Build
cargo build --release

# Test
php -dextension=target/release/libext_shopware.so -m
```

See the main [README.md](README.md) for more details on local development.
