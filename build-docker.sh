#!/bin/bash
set -e

VERSIONS=("8.1" "8.2" "8.3")
ARCHS=("gnu" "musl")

mkdir -p output

for version in "${VERSIONS[@]}"; do
    for arch in "${ARCHS[@]}"; do
        echo "Building for PHP $version ($arch)..."
        
        # Build the image and export the artifact
        # We use --output type=local to extract files from the build
        DOCKER_BUILDKIT=1 docker build \
            --file Dockerfile.$arch \
            --build-arg PHP_VERSION=$version \
            --output type=local,dest=output/$version/$arch \
            .
            
        # Rename for clarity
        mv output/$version/$arch/libext_shopware.so output/shopware-php-${version}-${arch}.so
        rm -rf output/$version
        
        echo "Built output/shopware-php-${version}-${arch}.so"
    done
done

echo "All builds complete. Artifacts are in output/"
