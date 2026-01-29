<?php

declare(strict_types=1);

namespace Shopware\PHPExtension\Tests;

use PHPUnit\Framework\TestCase;
use Shopware\PHPExtension\Wasmtime\Wasmtime;
use Shopware\PHPExtension\Wasmtime\WasmModule;
use Shopware\PHPExtension\Wasmtime\WasmInstance;

class WasmtimeTest extends TestCase
{
    private Wasmtime $wasm;

    protected function setUp(): void
    {
        $this->wasm = new Wasmtime();
    }

    public function testCanCreateInstance(): void
    {
        $this->assertInstanceOf(Wasmtime::class, $this->wasm);
    }

    public function testLoadModuleFromWat(): void
    {
        $wat = '(module)';
        $module = $this->wasm->loadModule($wat);
        $this->assertInstanceOf(WasmModule::class, $module);
    }

    public function testLoadModuleWithAddFunction(): void
    {
        $wat = '(module
            (func (export "add") (param i32 i32) (result i32)
                local.get 0
                local.get 1
                i32.add
            )
        )';
        $module = $this->wasm->loadModule($wat);
        $this->assertInstanceOf(WasmModule::class, $module);
    }

    public function testValidateModuleValid(): void
    {
        $wat = '(module)';
        $this->assertTrue(Wasmtime::validateModule($wat));
    }

    public function testValidateModuleInvalid(): void
    {
        $invalid = 'not valid wasm';
        $this->assertFalse(Wasmtime::validateModule($invalid));
    }

    public function testInstantiateModule(): void
    {
        $wat = '(module)';
        $module = $this->wasm->loadModule($wat);
        $instance = $this->wasm->instantiate($module);
        $this->assertInstanceOf(WasmInstance::class, $instance);
    }

    public function testLoadModuleFromFile(): void
    {
        $wat = '(module
            (func (export "double") (param i32) (result i32)
                local.get 0
                i32.const 2
                i32.mul
            )
        )';
        $tmpFile = sys_get_temp_dir() . '/test_module.wat';
        file_put_contents($tmpFile, $wat);

        try {
            $module = $this->wasm->loadModuleFromFile($tmpFile);
            $this->assertInstanceOf(WasmModule::class, $module);

            $instance = $this->wasm->instantiate($module);
            $result = $instance->call('double', [21]);
            $this->assertSame(42, $result);
        } finally {
            unlink($tmpFile);
        }
    }

    public function testLoadModuleFromFileNotFound(): void
    {
        $this->expectException(\Exception::class);
        $this->wasm->loadModuleFromFile('/nonexistent/path/to/module.wasm');
    }

    public function testCallAddFunctionI32(): void
    {
        $wat = '(module
            (func (export "add") (param i32 i32) (result i32)
                local.get 0
                local.get 1
                i32.add
            )
        )';
        $module = $this->wasm->loadModule($wat);
        $instance = $this->wasm->instantiate($module);

        $result = $instance->call('add', [40, 2]);
        $this->assertSame(42, $result);
    }

    public function testCallSubtractFunctionI32(): void
    {
        $wat = '(module
            (func (export "sub") (param i32 i32) (result i32)
                local.get 0
                local.get 1
                i32.sub
            )
        )';
        $module = $this->wasm->loadModule($wat);
        $instance = $this->wasm->instantiate($module);

        $result = $instance->call('sub', [50, 8]);
        $this->assertSame(42, $result);
    }

    public function testCallMultiplyFunctionI32(): void
    {
        $wat = '(module
            (func (export "mul") (param i32 i32) (result i32)
                local.get 0
                local.get 1
                i32.mul
            )
        )';
        $module = $this->wasm->loadModule($wat);
        $instance = $this->wasm->instantiate($module);

        $result = $instance->call('mul', [6, 7]);
        $this->assertSame(42, $result);
    }

    public function testCallFunctionI64(): void
    {
        $wat = '(module
            (func (export "add64") (param i64 i64) (result i64)
                local.get 0
                local.get 1
                i64.add
            )
        )';
        $module = $this->wasm->loadModule($wat);
        $instance = $this->wasm->instantiate($module);

        $result = $instance->call('add64', [1000000000000, 2000000000000]);
        $this->assertSame(3000000000000, $result);
    }

    public function testCallFunctionF32(): void
    {
        $wat = '(module
            (func (export "addf32") (param f32 f32) (result f32)
                local.get 0
                local.get 1
                f32.add
            )
        )';
        $module = $this->wasm->loadModule($wat);
        $instance = $this->wasm->instantiate($module);

        $result = $instance->call('addf32', [1.5, 2.5]);
        $this->assertEqualsWithDelta(4.0, $result, 0.001);
    }

    public function testCallFunctionF64(): void
    {
        $wat = '(module
            (func (export "addf64") (param f64 f64) (result f64)
                local.get 0
                local.get 1
                f64.add
            )
        )';
        $module = $this->wasm->loadModule($wat);
        $instance = $this->wasm->instantiate($module);

        $result = $instance->call('addf64', [3.14159, 2.71828]);
        $this->assertEqualsWithDelta(5.85987, $result, 0.00001);
    }

    public function testCallFunctionNoReturn(): void
    {
        $wat = '(module
            (func (export "noop"))
        )';
        $module = $this->wasm->loadModule($wat);
        $instance = $this->wasm->instantiate($module);

        $result = $instance->call('noop', []);
        $this->assertNull($result);
    }

    public function testCallNonexistentFunctionThrows(): void
    {
        $wat = '(module)';
        $module = $this->wasm->loadModule($wat);
        $instance = $this->wasm->instantiate($module);

        $this->expectException(\Exception::class);
        $instance->call('nonexistent', []);
    }

    public function testTooManyArgumentsThrows(): void
    {
        $wat = '(module
            (func (export "single") (param i32) (result i32)
                local.get 0
            )
        )';
        $module = $this->wasm->loadModule($wat);
        $instance = $this->wasm->instantiate($module);

        $this->expectException(\Exception::class);
        $instance->call('single', [1, 2, 3]);
    }

    public function testGetModuleExports(): void
    {
        $wat = '(module
            (func (export "add") (param i32 i32) (result i32)
                local.get 0
                local.get 1
                i32.add
            )
            (func (export "sub") (param i32 i32) (result i32)
                local.get 0
                local.get 1
                i32.sub
            )
        )';
        $module = $this->wasm->loadModule($wat);
        $exports = $module->getExports();

        $this->assertIsArray($exports);
        $this->assertCount(2, $exports);

        $names = array_column($exports, 'name');
        $this->assertContains('add', $names);
        $this->assertContains('sub', $names);

        foreach ($exports as $export) {
            $this->assertSame('function', $export['type']);
        }
    }

    public function testGetInstanceExports(): void
    {
        $wat = '(module
            (func (export "foo") (result i32)
                i32.const 42
            )
        )';
        $module = $this->wasm->loadModule($wat);
        $instance = $this->wasm->instantiate($module);
        $exports = $instance->getExports();

        $this->assertIsArray($exports);
        $this->assertGreaterThanOrEqual(1, count($exports));

        $names = array_column($exports, 'name');
        $this->assertContains('foo', $names);
    }

    public function testGlobalGetAndSet(): void
    {
        $wat = '(module
            (global (export "counter") (mut i32) (i32.const 0))
        )';
        $module = $this->wasm->loadModule($wat);
        $instance = $this->wasm->instantiate($module);

        $value = $instance->getGlobal('counter');
        $this->assertSame(0, $value);

        $instance->setGlobal('counter', 42);
        $value = $instance->getGlobal('counter');
        $this->assertSame(42, $value);
    }

    public function testImmutableGlobalThrowsOnSet(): void
    {
        $wat = '(module
            (global (export "constant") i32 (i32.const 100))
        )';
        $module = $this->wasm->loadModule($wat);
        $instance = $this->wasm->instantiate($module);

        $value = $instance->getGlobal('constant');
        $this->assertSame(100, $value);

        $this->expectException(\Exception::class);
        $instance->setGlobal('constant', 200);
    }

    public function testMemoryReadWrite(): void
    {
        $wat = '(module
            (memory (export "memory") 1)
        )';
        $module = $this->wasm->loadModule($wat);
        $instance = $this->wasm->instantiate($module);

        $instance->writeMemory(0, [0x48, 0x65, 0x6c, 0x6c, 0x6f]);
        $bytes = $instance->readMemory(0, 5);
        $this->assertSame([0x48, 0x65, 0x6c, 0x6c, 0x6f], $bytes);
    }

    public function testMemoryReadWriteInt32(): void
    {
        $wat = '(module
            (memory (export "memory") 1)
        )';
        $module = $this->wasm->loadModule($wat);
        $instance = $this->wasm->instantiate($module);

        $instance->writeInt32(0, 12345678);
        $value = $instance->readInt32(0);
        $this->assertSame(12345678, $value);
    }

    public function testMemoryReadWriteInt64(): void
    {
        $wat = '(module
            (memory (export "memory") 1)
        )';
        $module = $this->wasm->loadModule($wat);
        $instance = $this->wasm->instantiate($module);

        $instance->writeInt64(0, 1234567890123456789);
        $value = $instance->readInt64(0);
        $this->assertSame(1234567890123456789, $value);
    }

    public function testMemoryReadWriteFloat32(): void
    {
        $wat = '(module
            (memory (export "memory") 1)
        )';
        $module = $this->wasm->loadModule($wat);
        $instance = $this->wasm->instantiate($module);

        $instance->writeFloat32(0, 3.14159);
        $value = $instance->readFloat32(0);
        $this->assertEqualsWithDelta(3.14159, $value, 0.0001);
    }

    public function testMemoryReadWriteFloat64(): void
    {
        $wat = '(module
            (memory (export "memory") 1)
        )';
        $module = $this->wasm->loadModule($wat);
        $instance = $this->wasm->instantiate($module);

        $instance->writeFloat64(0, 3.141592653589793);
        $value = $instance->readFloat64(0);
        $this->assertEqualsWithDelta(3.141592653589793, $value, 0.0000000001);
    }

    public function testMemoryReadWriteString(): void
    {
        $wat = '(module
            (memory (export "memory") 1)
        )';
        $module = $this->wasm->loadModule($wat);
        $instance = $this->wasm->instantiate($module);

        $instance->writeString(0, 'Hello, WebAssembly!');
        $value = $instance->readString(0, 19);
        $this->assertSame('Hello, WebAssembly!', $value);
    }

    public function testMemorySize(): void
    {
        $wat = '(module
            (memory (export "memory") 2)
        )';
        $module = $this->wasm->loadModule($wat);
        $instance = $this->wasm->instantiate($module);

        $size = $instance->memorySize();
        $this->assertSame(131072, $size);
    }

    public function testMemoryOutOfBoundsReadThrows(): void
    {
        $wat = '(module
            (memory (export "memory") 1)
        )';
        $module = $this->wasm->loadModule($wat);
        $instance = $this->wasm->instantiate($module);

        $this->expectException(\Exception::class);
        $instance->readMemory(65536, 1);
    }

    public function testMemoryOutOfBoundsWriteThrows(): void
    {
        $wat = '(module
            (memory (export "memory") 1)
        )';
        $module = $this->wasm->loadModule($wat);
        $instance = $this->wasm->instantiate($module);

        $this->expectException(\Exception::class);
        $instance->writeMemory(65536, [0x00]);
    }

    public function testFuelEnabled(): void
    {
        $wasm = new Wasmtime();
        $wasm->setFuelEnabled(true);
        $wasm->setFuel(1000000);

        $wat = '(module
            (func (export "compute") (result i32)
                (local $i i32)
                (local.set $i (i32.const 0))
                (block $break
                    (loop $continue
                        (local.set $i (i32.add (local.get $i) (i32.const 1)))
                        (br_if $break (i32.ge_u (local.get $i) (i32.const 100)))
                        (br $continue)
                    )
                )
                (local.get $i)
            )
        )';

        $module = $wasm->loadModule($wat);
        $instance = $wasm->instantiate($module);

        $result = $instance->call('compute', []);
        $this->assertSame(100, $result);
    }

    public function testInstanceIsolation(): void
    {
        $wat = '(module
            (global (export "counter") (mut i32) (i32.const 0))
        )';
        $module = $this->wasm->loadModule($wat);

        $instance1 = $this->wasm->instantiate($module);
        $instance2 = $this->wasm->instantiate($module);

        $instance1->setGlobal('counter', 100);
        $instance2->setGlobal('counter', 200);

        $this->assertSame(100, $instance1->getGlobal('counter'));
        $this->assertSame(200, $instance2->getGlobal('counter'));
    }

    public function testLoadInvalidModuleThrows(): void
    {
        $this->expectException(\Exception::class);
        $this->wasm->loadModule('invalid wasm bytes');
    }

    public function testCallWithBooleanArgument(): void
    {
        $wat = '(module
            (func (export "identity") (param i32) (result i32)
                local.get 0
            )
        )';
        $module = $this->wasm->loadModule($wat);
        $instance = $this->wasm->instantiate($module);

        $result = $instance->call('identity', [true]);
        $this->assertSame(1, $result);

        $result = $instance->call('identity', [false]);
        $this->assertSame(0, $result);
    }

    public function testCallWithFloatToIntConversion(): void
    {
        $wat = '(module
            (func (export "identity") (param i32) (result i32)
                local.get 0
            )
        )';
        $module = $this->wasm->loadModule($wat);
        $instance = $this->wasm->instantiate($module);

        $result = $instance->call('identity', [42.7]);
        $this->assertSame(42, $result);
    }

    public function testMemoryWithFunctions(): void
    {
        $wat = '(module
            (memory (export "memory") 1)
            (func (export "store") (param i32 i32)
                local.get 0
                local.get 1
                i32.store
            )
            (func (export "load") (param i32) (result i32)
                local.get 0
                i32.load
            )
        )';
        $module = $this->wasm->loadModule($wat);
        $instance = $this->wasm->instantiate($module);

        $instance->call('store', [0, 42]);
        $result = $instance->call('load', [0]);
        $this->assertSame(42, $result);

        $value = $instance->readInt32(0);
        $this->assertSame(42, $value);
    }

    public function testComplexWasmModule(): void
    {
        $wat = '(module
            (func (export "factorial") (param i64) (result i64)
                (if (result i64) (i64.le_s (local.get 0) (i64.const 1))
                    (then (i64.const 1))
                    (else
                        (i64.mul
                            (local.get 0)
                            (call 0 (i64.sub (local.get 0) (i64.const 1)))
                        )
                    )
                )
            )
        )';
        $module = $this->wasm->loadModule($wat);
        $instance = $this->wasm->instantiate($module);

        $this->assertSame(1, $instance->call('factorial', [0]));
        $this->assertSame(1, $instance->call('factorial', [1]));
        $this->assertSame(2, $instance->call('factorial', [2]));
        $this->assertSame(6, $instance->call('factorial', [3]));
        $this->assertSame(24, $instance->call('factorial', [4]));
        $this->assertSame(120, $instance->call('factorial', [5]));
        $this->assertSame(3628800, $instance->call('factorial', [10]));
    }
}
