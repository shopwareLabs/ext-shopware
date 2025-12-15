# Shopware PHP Extension

A high-performance PHP extension written in Rust, providing bindings for:

- **QuickJS** - Fast JavaScript engine for executing JS from PHP
- **LightningCSS** - Fast CSS parser, transformer, and minifier
- **libvips** - Fast image processing library

## Pre-built Binaries

Pre-built binaries are automatically generated for:
- **Architectures**: x86_64 (amd64), arm64
- **PHP Versions**: 8.1, 8.2, 8.3, 8.4
- **Systems**: Alpine Linux (musl libc), Debian/Ubuntu (glibc)

Check the [GitHub Actions artifacts](../../actions/workflows/build-multi-arch.yml) for the latest builds.

## Requirements

- PHP 8.1+
- Rust (latest stable)
- System dependencies:
  - libvips (for image processing)
  - pkg-config

### macOS

```bash
brew install vips pkg-config
```

### Ubuntu/Debian

```bash
sudo apt-get install libvips-dev pkg-config
```

## Building

### Local Development

```bash
# Install dependencies
composer install

# Build release version
make build

# Build debug version
make build-debug
```

### Multi-Architecture Builds with Docker

For building across multiple architectures and PHP versions, see [BUILD.md](BUILD.md) for detailed instructions on:
- Building for x86_64 and arm64 architectures
- Building for PHP 8.1, 8.2, 8.3, and 8.4
- Building for musl (Alpine) and glibc (Debian) based systems
- Using Docker for local testing

Quick start:
```bash
# Build for PHP 8.4 on Debian x86_64
./build-local.sh 8.4 bookworm x86_64

# Build for PHP 8.3 on Alpine arm64
./build-local.sh 8.3 alpine3.20 arm64
```

## Running Tests

```bash
# Run tests with release build
make test

# Run tests with debug build
make test-debug
```

## Running Benchmarks

```bash
make bench
```

## PHP Classes

### QuickJS - JavaScript Engine

Execute JavaScript code from PHP with full bidirectional data exchange.

```php
use Shopware\PHPExtension\QuickJS\QuickJS;
use Shopware\PHPExtension\QuickJS\QuickObject;

$js = new QuickJS();

// Evaluate JavaScript code
$result = $js->eval('1 + 2'); // Returns: 3

// Set global variables
$js->setGlobal('name', 'World');
$js->setGlobal('config', ['debug' => true, 'version' => '1.0.0']);
echo $js->eval("'Hello, ' + name"); // Returns: "Hello, World"

// Get global variables
$js->eval("var counter = 42");
$value = $js->getGlobal('counter'); // Returns: 42

// Check if global exists
$js->hasGlobal('counter'); // Returns: true

// Get type of global
$js->typeofGlobal('counter'); // Returns: "number"

// Call JavaScript functions
$js->eval("function greet(name) { return 'Hello, ' + name + '!'; }");
$result = $js->call('greet', ['World']); // Returns: "Hello, World!"

// Register PHP functions for use in JavaScript
$js->registerFunction('upper', 'strtoupper');
$js->registerFunction('md5', 'md5');
echo $js->eval("upper('hello')"); // Returns: "HELLO"

// Create JavaScript objects with QuickObject
$obj = new QuickObject();
$obj->registerProperty('version', '1.0.0');
$obj->registerProperty('debug', true);
$obj->registerFunction('hash', 'md5');

$js->registerObject('App', $obj);
echo $js->eval("App.version"); // Returns: "1.0.0"
echo $js->eval("App.hash('test')"); // Returns: md5 hash

// Nested objects
$db = new QuickObject();
$db->registerProperty('host', 'localhost');
$db->registerProperty('port', 3306);

$config = new QuickObject();
$config->registerProperty('name', 'MyApp');
$config->registerObject('database', $db);

$js->registerObject('Config', $config);
echo $js->eval("Config.database.host"); // Returns: "localhost"

// Memory management
$js->setMemoryLimit(10 * 1024 * 1024); // 10MB limit
$js->setMaxStackSize(1024 * 1024); // 1MB stack
$js->gc(); // Run garbage collection
$usage = $js->memoryUsage(); // Get memory usage in bytes
```

### LightningCSS - CSS Processing

Fast CSS parsing, minification, and transformation.

```php
use Shopware\PHPExtension\LightningCSS\LightningCSS;

$css = new LightningCSS();

// Minify CSS
$minified = $css->minify('body { color: red; }');
// Returns: "body{color:red}"

// Format/pretty-print CSS
$formatted = $css->format('body{color:red}');
// Returns formatted CSS with newlines

// Validate CSS syntax
$isValid = $css->validate('body { color: red; }'); // Returns: true

// Analyze CSS
$info = $css->analyze('body { color: red; } .test { margin: 10px; }');
// Returns: ['rules_count' => 2]

// Set browser targets for compatibility
$css->setBrowserTargets([
    'chrome' => 95,
    'firefox' => 90,
    'safari' => 14,
    'edge' => 95,
    'ie' => 11,
    'opera' => 80,
    'ios_safari' => 14,
    'android' => 95,
    'samsung' => 15,
]);

// Transform CSS with vendor prefixes for older browsers
$transformed = $css->transform('.test { user-select: none; }');
```

### Image - Image Processing

Fast image processing powered by libvips.

```php
use Shopware\PHPExtension\Image\Image;

// Check supported formats
$formats = Image::getSupportedFormats();
// Returns: ['jpeg' => true, 'png' => true, 'webp' => true, 'jxl' => true, 'avif' => false, ...]

if (Image::supportsFormat('avif')) {
    // AVIF is supported
}

// Load image from file
$img = Image::fromFile('/path/to/image.jpg');

// Load image from string/buffer
$data = file_get_contents('/path/to/image.jpg');
$img = Image::fromString($data);

// Get dimensions
$dimensions = $img->getDimension();
// Returns: ['width' => 800, 'height' => 600]

// Resize image (maintains aspect ratio)
$resized = $img->resize(400, 300);

// Save to file
$img->saveFile('/path/to/output.jpg');

// Save with specific format
$img->saveFile('/path/to/output.webp', 'webp');

// Save with quality setting (1-100)
$img->saveFile('/path/to/output.jpg', 'jpeg', 85);

// Save to string/buffer
$buffer = $img->saveString('png');
$buffer = $img->saveString('jpeg', 90);

// Format constants
Image::FORMAT_JPEG; // 'jpeg'
Image::FORMAT_PNG;  // 'png'
Image::FORMAT_WEBP; // 'webp'
Image::FORMAT_TIFF; // 'tiff'
Image::FORMAT_JXL;  // 'jxl'
Image::FORMAT_AVIF; // 'avif'
```

## License

MIT
