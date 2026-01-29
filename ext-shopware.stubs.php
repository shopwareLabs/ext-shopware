<?php

// Stubs for ext-shopware

namespace Shopware\PHPExtension\LightningCSS {
    /**
     * LightningCSS - A fast CSS parser, transformer, and minifier
     *
     * Example usage:
     * ```php
     * $css = new LightningCSS();
     * $result = $css->minify('body { color: red; }');
     * ```
     */
    class LightningCSS {
        /**
         * Set browser targets for compatibility transformations
         *
         * Example:
         * ```php
         * $css->setBrowserTargets([
         *     'chrome' => 95,
         *     'firefox' => 90,
         *     'safari' => 14,
         * ]);
         * ```
         */
        public function setBrowserTargets(array $browsers): mixed {}

        /**
         * Minify CSS code
         *
         * Returns minified CSS string
         */
        public function minify(string $css): string {}

        /**
         * Transform CSS for browser compatibility without minification
         *
         * Adds vendor prefixes and transforms modern syntax for older browsers
         */
        public function transform(string $css): string {}

        /**
         * Parse and pretty-print CSS (formats the CSS)
         */
        public function format(string $css): string {}

        /**
         * Validate CSS syntax
         *
         * Returns true if CSS is valid, throws exception with details if invalid
         */
        public function validate(string $css): bool {}

        /**
         * Parse CSS and return analysis information
         *
         * Returns an array with information about the stylesheet
         */
        public function analyze(string $css): mixed {}

        /**
         * Create a new LightningCSS instance
         */
        public function __construct() {}
    }
}

namespace Shopware\PHPExtension\QuickJS {
    /**
     * QuickJS JavaScript engine class for PHP
     * 
     * This class provides a simple interface to execute JavaScript code from PHP.
     * It combines runtime and context management into a single, easy-to-use class.
     */
    class QuickJS {
        /**
         * Set memory limit in bytes
         */
        public function setMemoryLimit(int $limit): mixed {}

        /**
         * Set max stack size in bytes
         */
        public function setMaxStackSize(int $size): mixed {}

        /**
         * Run garbage collection
         */
        public function gc(): mixed {}

        /**
         * Get memory usage in bytes
         */
        public function memoryUsage(): int {}

        /**
         * Evaluate JavaScript code and return the result
         */
        public function eval(string $code): mixed {}

        /**
         * Evaluate JavaScript code from a file
         */
        public function evalFile(string $filename): mixed {}

        /**
         * Set a global variable in the JavaScript context
         */
        public function setGlobal(string $name, mixed $value): mixed {}

        /**
         * Get a global variable from the JavaScript context
         */
        public function getGlobal(string $name): mixed {}

        /**
         * Call a JavaScript function by name with arguments
         */
        public function call(string $function_name, array $args): mixed {}

        /**
         * Check if a global variable exists
         */
        public function hasGlobal(string $name): bool {}

        /**
         * Get the type of a global variable as a string
         */
        public function typeofGlobal(string $name): string {}

        /**
         * Register a PHP function (by name) as a JavaScript function
         * Pass the function name as a string, e.g., "strtoupper", "array_sum"
         */
        public function registerFunction(string $js_name, string $php_function_name): mixed {}

        /**
         * Register a global JavaScript object using a QuickObject
         * 
         * Example:
         * ```php
         * $obj = new QuickObject();
         * $obj->registerProperty('version', '1.0.0');
         * $obj->registerFunction('upper', 'strtoupper');
         * $js->registerObject('MyApp', $obj);
         * ```
         */
        public function registerObject(string $js_name, \Shopware\PHPExtension\QuickJS\QuickObject $obj): mixed {}

        /**
         * Create or get a global object and add a method to it
         * This allows building objects incrementally
         */
        public function addObjectMethod(string $object_name, string $method_name, string $php_function_name): mixed {}

        /**
         * Add a property to an existing or new global object
         */
        public function addObjectProperty(string $object_name, string $property_name, mixed $value): mixed {}

        /**
         * Create a new QuickJS instance
         */
        public function __construct() {}
    }

    /**
     * QuickObject - A builder class for creating JavaScript objects
     * 
     * Example usage:
     * ```php
     * $obj = new QuickObject();
     * $obj->registerProperty('version', '1.0.0');
     * $obj->registerFunction('upper', 'strtoupper');
     * 
     * $nested = new QuickObject();
     * $nested->registerProperty('host', 'localhost');
     * $obj->registerObject('config', $nested);
     * 
     * $js->registerObject('MyApp', $obj);
     * ```
     */
    class QuickObject {
        /**
         * Register a property with a value
         */
        public function registerProperty(string $name, mixed $value): mixed {}

        /**
         * Register a PHP function as a method
         */
        public function registerFunction(string $name, string $php_function_name): mixed {}

        /**
         * Register a nested QuickObject
         */
        public function registerObject(string $name, \Shopware\PHPExtension\QuickJS\QuickObject $obj): mixed {}

        /**
         * Create a new QuickObject
         */
        public function __construct() {}
    }
}

namespace Shopware\PHPExtension\Wasmtime {
    /**
     * Wasmtime - WebAssembly runtime for PHP
     *
     * Example usage:
     * ```php
     * $wasm = new Wasmtime();
     * $module = $wasm->loadModule($wasmBytes);
     * $instance = $wasm->instantiate($module);
     * $result = $instance->call('add', [1, 2]);
     * ```
     */
    class Wasmtime {
        /**
         * Enable or disable fuel metering for execution limits
         * Note: This must be called before loadModule() for it to take effect
         */
        public function setFuelEnabled(bool $enabled): mixed {}

        /**
         * Set the fuel amount for execution limits
         */
        public function setFuel(int $amount): mixed {}

        /**
         * Register a PHP function to be callable from WASM as a host function
         *
         * Example:
         * ```php
         * $wasm->registerFunction('env', 'log', 'my_log_function');
         * ```
         */
        public function registerFunction(string $module_name, string $func_name, string $php_function_name): mixed {}

        /**
         * Load a WASM module from bytes (binary or WAT text format)
         * Pass a PHP string containing the WASM bytes or WAT text
         */
        public function loadModule(string $bytes): \Shopware\PHPExtension\Wasmtime\WasmModule {}

        /**
         * Load a WASM module from a file
         */
        public function loadModuleFromFile(string $path): \Shopware\PHPExtension\Wasmtime\WasmModule {}

        /**
         * Validate WASM module bytes without loading
         */
        public static function validateModule(string $bytes): bool {}

        /**
         * Instantiate a module with optional host functions
         */
        public function instantiate(\Shopware\PHPExtension\Wasmtime\WasmModule $module): \Shopware\PHPExtension\Wasmtime\WasmInstance {}

        /**
         * Create a new Wasmtime instance
         */
        public function __construct() {}
    }

    /**
     * Compiled WebAssembly module
     */
    class WasmModule {
        /**
         * Get the exports of this module
         */
        public function getExports(): mixed {}

        public function __construct() {}
    }

    /**
     * Instantiated WebAssembly module instance
     */
    class WasmInstance {
        /**
         * Call an exported function by name
         */
        public function call(string $name, array $args): mixed {}

        /**
         * Get the exports of this instance
         */
        public function getExports(): mixed {}

        /**
         * Get the value of a global export
         */
        public function getGlobal(string $name): mixed {}

        /**
         * Set the value of a mutable global export
         */
        public function setGlobal(string $name, mixed $value): mixed {}

        /**
         * Read bytes from memory at the given offset
         */
        public function readMemory(int $offset, int $length): array {}

        /**
         * Write bytes to memory at the given offset
         */
        public function writeMemory(int $offset, array $bytes): mixed {}

        /**
         * Read an i32 from memory at the given offset
         */
        public function readInt32(int $offset): int {}

        /**
         * Read an i64 from memory at the given offset
         */
        public function readInt64(int $offset): int {}

        /**
         * Read an f32 from memory at the given offset
         */
        public function readFloat32(int $offset): float {}

        /**
         * Read an f64 from memory at the given offset
         */
        public function readFloat64(int $offset): float {}

        /**
         * Read a UTF-8 string from memory at the given offset with the given length
         */
        public function readString(int $offset, int $length): string {}

        /**
         * Write an i32 to memory at the given offset
         */
        public function writeInt32(int $offset, int $value): mixed {}

        /**
         * Write an i64 to memory at the given offset
         */
        public function writeInt64(int $offset, int $value): mixed {}

        /**
         * Write an f32 to memory at the given offset
         */
        public function writeFloat32(int $offset, float $value): mixed {}

        /**
         * Write an f64 to memory at the given offset
         */
        public function writeFloat64(int $offset, float $value): mixed {}

        /**
         * Write a UTF-8 string to memory at the given offset
         */
        public function writeString(int $offset, string $value): mixed {}

        /**
         * Get the size of the memory in bytes
         */
        public function memorySize(): int {}

        public function __construct() {}
    }
}
