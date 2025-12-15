#!/bin/bash
# Local build script for ext-shopware
# Usage: ./build-local.sh [PHP_VERSION] [BASE_IMAGE] [ARCH]
# Example: ./build-local.sh 8.4 bookworm x86_64
# Example: ./build-local.sh 8.3 alpine3.20 arm64

set -e

# Default values
PHP_VERSION=${1:-8.4}
BASE_IMAGE=${2:-bookworm}
ARCH=${3:-x86_64}

# Convert architecture naming
DOCKER_ARCH="amd64"
if [ "$ARCH" = "arm64" ]; then
    DOCKER_ARCH="arm64"
fi

# Determine libc type from base image
LIBC="glibc"
if [[ "$BASE_IMAGE" == alpine* ]]; then
    LIBC="musl"
fi

echo "=========================================="
echo "Building ext-shopware with:"
echo "  PHP Version: $PHP_VERSION"
echo "  Base Image: $BASE_IMAGE"
echo "  Architecture: $ARCH ($DOCKER_ARCH)"
echo "  Libc: $LIBC"
echo "=========================================="

# Create output directory
mkdir -p ./build-output

# Build the extension
docker buildx build \
    --platform linux/$DOCKER_ARCH \
    --build-arg PHP_VERSION=$PHP_VERSION \
    --build-arg BASE_IMAGE=$BASE_IMAGE \
    --target builder \
    --output type=local,dest=./build-output \
    .

# Find and copy the built extension
echo ""
echo "Locating built extension..."
SO_FILE=$(find ./build-output -name "libext_shopware.so" -type f | head -n 1)

if [ -z "$SO_FILE" ]; then
    echo "ERROR: Could not find libext_shopware.so"
    exit 1
fi

# Create artifacts directory and copy with descriptive name
mkdir -p ./artifacts
ARTIFACT_NAME="libext_shopware-php${PHP_VERSION}-${ARCH}-${LIBC}.so"
cp "$SO_FILE" "./artifacts/$ARTIFACT_NAME"

echo ""
echo "=========================================="
echo "Build successful!"
echo "Extension saved to: ./artifacts/$ARTIFACT_NAME"
echo "=========================================="
echo ""
echo "To test the extension, run:"
echo "  docker run --rm -v \$(pwd)/artifacts:/ext php:${PHP_VERSION}-cli-${BASE_IMAGE} sh -c 'php -dextension=/ext/$ARTIFACT_NAME -m | grep -i shopware || echo \"ERROR: Extension failed to load\"'"
echo ""
echo "To build the full image with tests:"
echo "  docker buildx build --platform linux/$DOCKER_ARCH --build-arg PHP_VERSION=$PHP_VERSION --build-arg BASE_IMAGE=$BASE_IMAGE -t ext-shopware:test ."
echo "  docker run --rm ext-shopware:test php -m"
echo ""
