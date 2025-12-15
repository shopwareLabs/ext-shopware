# Using Pre-built Artifacts

This document explains how to download and use the pre-built extension binaries from GitHub Actions.

## Available Artifacts

After each successful build, the following artifacts are available for download:

### Naming Convention

Artifacts are named as:
```
ext-shopware-php{VERSION}-{ARCH}-{LIBC}
```

Examples:
- `ext-shopware-php8.4-x86_64-glibc` - PHP 8.4 on Debian/Ubuntu (x86_64)
- `ext-shopware-php8.3-arm64-musl` - PHP 8.3 on Alpine (ARM64)
- `ext-shopware-php8.1-x86_64-glibc` - PHP 8.1 on Debian/Ubuntu (x86_64)

### Matrix of Available Builds

| PHP Version | Architecture | Libc  | Artifact Name                          |
|-------------|--------------|-------|----------------------------------------|
| 8.1         | x86_64       | glibc | ext-shopware-php8.1-x86_64-glibc      |
| 8.1         | x86_64       | musl  | ext-shopware-php8.1-x86_64-musl       |
| 8.1         | arm64        | glibc | ext-shopware-php8.1-arm64-glibc       |
| 8.1         | arm64        | musl  | ext-shopware-php8.1-arm64-musl        |
| 8.2         | x86_64       | glibc | ext-shopware-php8.2-x86_64-glibc      |
| 8.2         | x86_64       | musl  | ext-shopware-php8.2-x86_64-musl       |
| 8.2         | arm64        | glibc | ext-shopware-php8.2-arm64-glibc       |
| 8.2         | arm64        | musl  | ext-shopware-php8.2-arm64-musl        |
| 8.3         | x86_64       | glibc | ext-shopware-php8.3-x86_64-glibc      |
| 8.3         | x86_64       | musl  | ext-shopware-php8.3-x86_64-musl       |
| 8.3         | arm64        | glibc | ext-shopware-php8.3-arm64-glibc       |
| 8.3         | arm64        | musl  | ext-shopware-php8.3-arm64-musl        |
| 8.4         | x86_64       | glibc | ext-shopware-php8.4-x86_64-glibc      |
| 8.4         | x86_64       | musl  | ext-shopware-php8.4-x86_64-musl       |
| 8.4         | arm64        | glibc | ext-shopware-php8.4-arm64-glibc       |
| 8.4         | arm64        | musl  | ext-shopware-php8.4-arm64-musl        |

## Downloading Artifacts

### From GitHub UI

1. Go to the [Actions tab](../../actions/workflows/build-multi-arch.yml)
2. Click on a successful workflow run (green checkmark)
3. Scroll to the "Artifacts" section at the bottom
4. Download the artifact for your target platform

### Using GitHub CLI

```bash
# List available artifacts from the latest run
gh run list --workflow=build-multi-arch.yml --limit 1

# Download a specific artifact
gh run download --name ext-shopware-php8.4-x86_64-glibc
```

## Installing the Extension

### On Debian/Ubuntu (glibc)

```bash
# Download and extract the artifact
# The .so file will be named like: libext_shopware-php8.4-x86_64-glibc.so

# Install runtime dependencies
sudo apt-get install libvips

# Copy to PHP extensions directory
sudo cp libext_shopware-php8.4-x86_64-glibc.so /usr/lib/php/$(php -r 'echo PHP_MAJOR_VERSION.".".PHP_MINOR_VERSION;')/

# Enable the extension
echo "extension=libext_shopware-php8.4-x86_64-glibc.so" | sudo tee /etc/php/$(php -r 'echo PHP_MAJOR_VERSION.".".PHP_MINOR_VERSION;')/mods-available/shopware.ini

# Enable it for CLI
sudo phpenmod shopware

# Verify
php -m | grep -i shopware
```

### On Alpine Linux (musl)

```bash
# Download and extract the artifact

# Install runtime dependencies
apk add vips

# Copy to PHP extensions directory
cp libext_shopware-php8.4-x86_64-musl.so /usr/lib/php*/modules/

# Enable the extension
echo "extension=libext_shopware-php8.4-x86_64-musl.so" > /etc/php*/conf.d/shopware.ini

# Verify
php -m | grep -i shopware
```

### Using Docker

The easiest way to use the pre-built extension:

```dockerfile
FROM php:8.4-cli-bookworm

# Install runtime dependencies
RUN apt-get update && \
    apt-get install -y libvips && \
    rm -rf /var/lib/apt/lists/*

# Copy the pre-built extension
COPY libext_shopware-php8.4-x86_64-glibc.so /usr/local/lib/php/extensions/

# Enable the extension
RUN echo "extension=/usr/local/lib/php/extensions/libext_shopware-php8.4-x86_64-glibc.so" > \
    /usr/local/etc/php/conf.d/ext-shopware.ini

# Verify
RUN php -m | grep -i shopware
```

## Choosing the Right Artifact

### Determine your architecture:

```bash
uname -m
# x86_64 or aarch64 (arm64)
```

### Determine your libc:

```bash
ldd --version
# If it shows "GNU libc" or "GLIBC", use glibc variants
# If it shows "musl libc", use musl variants
```

### Determine your PHP version:

```bash
php -v
```

## Compatibility

- **glibc builds**: For Debian, Ubuntu, CentOS, RHEL, Fedora, and most standard Linux distributions
- **musl builds**: For Alpine Linux and other musl-based distributions
- **Architecture**: Ensure the architecture matches your system (x86_64 vs arm64)
- **PHP version**: The extension must match your PHP major.minor version exactly

## Troubleshooting

### Extension not loading

Check PHP error logs:
```bash
php -d display_errors=1 -d display_startup_errors=1 -r "echo 'test';"
```

### Missing libvips

```bash
# Debian/Ubuntu
sudo apt-get install libvips

# Alpine
apk add vips

# Verify
ldconfig -p | grep vips
```

### Architecture mismatch

```bash
file libext_shopware-php8.4-x86_64-glibc.so
# Should show: ELF 64-bit LSB shared object, x86-64
```

Make sure it matches your system architecture.
