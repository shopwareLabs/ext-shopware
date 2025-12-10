<?php

declare(strict_types=1);

namespace Shopware\PHPExtension\Tests;

use PHPUnit\Framework\TestCase;
use Shopware\PHPExtension\QuickJS\QuickJS;
use Shopware\PHPExtension\QuickJS\QuickObject;

class QuickJSTest extends TestCase
{
    private QuickJS $js;

    protected function setUp(): void
    {
        $this->js = new QuickJS();
    }

    public function testCanCreateInstance(): void
    {
        $this->assertInstanceOf(QuickJS::class, $this->js);
    }

    public function testEvalBasicExpression(): void
    {
        $result = $this->js->eval("1 + 2");
        $this->assertSame(3, $result);
    }

    public function testEvalThrowsOnSyntaxError(): void
    {
        $this->expectException(\Exception::class);
        $this->js->eval("invalid javascript code here");
    }

    public function testSetAndGetGlobalString(): void
    {
        $this->js->setGlobal('name', 'John');
        $result = $this->js->eval("name");
        $this->assertSame('John', $result);
    }

    public function testSetAndGetGlobalNumber(): void
    {
        $this->js->setGlobal('count', 42);
        $result = $this->js->eval("count * 2");
        $this->assertSame(84, $result);
    }

    public function testSetAndGetGlobalArray(): void
    {
        $this->js->setGlobal('items', [1, 2, 3]);
        $result = $this->js->eval("items.reduce((a, b) => a + b, 0)");
        $this->assertSame(6, $result);
    }

    public function testSetAndGetGlobalObject(): void
    {
        $this->js->setGlobal('config', [
            'debug' => true,
            'version' => '1.0.0',
        ]);

        $this->assertSame('1.0.0', $this->js->eval("config.version"));
        $this->assertTrue($this->js->eval("config.debug"));
    }

    public function testGetGlobal(): void
    {
        $this->js->eval("var globalVar = 'test value'");
        $result = $this->js->getGlobal('globalVar');
        $this->assertSame('test value', $result);
    }

    public function testHasGlobalReturnsTrue(): void
    {
        $this->js->setGlobal('exists', true);
        $this->assertTrue($this->js->hasGlobal('exists'));
    }

    public function testHasGlobalReturnsFalse(): void
    {
        $this->assertFalse($this->js->hasGlobal('nonexistent'));
    }

    public function testTypeofGlobalFunction(): void
    {
        $this->js->eval("function testFunc() {}");
        $this->assertSame('function', $this->js->typeofGlobal('testFunc'));
    }

    public function testTypeofGlobalNumber(): void
    {
        $this->js->setGlobal('num', 42);
        $this->assertSame('number', $this->js->typeofGlobal('num'));
    }

    public function testTypeofGlobalString(): void
    {
        $this->js->setGlobal('str', 'hello');
        $this->assertSame('string', $this->js->typeofGlobal('str'));
    }

    public function testTypeofGlobalObject(): void
    {
        $this->js->setGlobal('obj', ['key' => 'value']);
        $this->assertSame('object', $this->js->typeofGlobal('obj'));
    }

    public function testTypeofGlobalBoolean(): void
    {
        $this->js->setGlobal('flag', true);
        $this->assertSame('boolean', $this->js->typeofGlobal('flag'));
    }

    public function testCallJavaScriptFunction(): void
    {
        $this->js->eval("function greet(name) { return 'Hello, ' + name + '!'; }");
        $result = $this->js->call('greet', ['World']);
        $this->assertSame('Hello, World!', $result);
    }

    public function testCallJavaScriptFunctionWithMultipleArgs(): void
    {
        $this->js->eval("function add(a, b) { return a + b; }");
        $result = $this->js->call('add', [10, 20]);
        $this->assertSame(30, $result);
    }

    public function testCallJavaScriptFunctionWithArrayArg(): void
    {
        $this->js->eval("function sum(arr) { return arr.reduce((a, b) => a + b, 0); }");
        $result = $this->js->call('sum', [[1, 2, 3, 4, 5]]);
        $this->assertSame(15, $result);
    }

    public function testCallNonexistentFunctionThrows(): void
    {
        $this->expectException(\Exception::class);
        $this->js->call('nonexistent', []);
    }

    public function testContextMaintainsState(): void
    {
        $this->js->eval("let counter = 0");
        $this->js->eval("counter++");
        $this->js->eval("counter++");
        $this->js->eval("counter++");

        $result = $this->js->eval("counter");
        $this->assertSame(3, $result);
    }

    public function testInstanceIsolation(): void
    {
        $js1 = new QuickJS();
        $js2 = new QuickJS();

        $js1->setGlobal('value', 'instance1');
        $js2->setGlobal('value', 'instance2');

        $this->assertSame('instance1', $js1->eval('value'));
        $this->assertSame('instance2', $js2->eval('value'));
    }

    public function testSetMemoryLimit(): void
    {
        $this->js->setMemoryLimit(10 * 1024 * 1024);

        $result = $this->js->eval("'test'");
        $this->assertSame('test', $result);
    }

    public function testSetMaxStackSize(): void
    {
        $this->js->setMaxStackSize(1024 * 1024);

        $result = $this->js->eval("'test'");
        $this->assertSame('test', $result);
    }

    public function testMemoryUsageReturnsPositiveValue(): void
    {
        $this->js->eval("const data = [1, 2, 3, 4, 5]");

        $usage = $this->js->memoryUsage();
        $this->assertIsInt($usage);
        $this->assertGreaterThan(0, $usage);
    }

    public function testGcDoesNotThrow(): void
    {
        $this->js->eval("const data = [1, 2, 3]");

        $this->js->gc();

        $result = $this->js->eval("data.length");
        $this->assertSame(3, $result);
    }

    public function testRegisterBuiltinFunction(): void
    {
        $this->js->registerFunction('php_strtoupper', 'strtoupper');

        $result = $this->js->eval("php_strtoupper('hello')");
        $this->assertSame('HELLO', $result);
    }

    public function testRegisterMultipleBuiltinFunctions(): void
    {
        $this->js->registerFunction('upper', 'strtoupper');
        $this->js->registerFunction('lower', 'strtolower');
        $this->js->registerFunction('len', 'strlen');

        $this->assertSame('HELLO', $this->js->eval("upper('hello')"));
        $this->assertSame('hello', $this->js->eval("lower('HELLO')"));
        $this->assertSame(5, $this->js->eval("len('hello')"));
    }

    public function testRegisterFunctionWithMultipleArgs(): void
    {
        $this->js->registerFunction('repeat', 'str_repeat');

        $result = $this->js->eval("repeat('ab', 3)");
        $this->assertSame('ababab', $result);
    }

    public function testRegisterArrayFunction(): void
    {
        $this->js->registerFunction('sum', 'array_sum');

        $result = $this->js->eval("sum([1, 2, 3, 4, 5])");
        $this->assertSame(15, $result);
    }

    public function testRegisterImplodeFunction(): void
    {
        $this->js->registerFunction('implode', 'implode');

        $result = $this->js->eval("implode('-', ['a', 'b', 'c'])");
        $this->assertSame('a-b-c', $result);
    }

    public function testRegisterMd5Function(): void
    {
        $this->js->registerFunction('md5', 'md5');

        $result = $this->js->eval("md5('password')");
        $this->assertSame('5f4dcc3b5aa765d61d8327deb882cf99', $result);
    }

    public function testUseFunctionInJavaScriptExpression(): void
    {
        $this->js->registerFunction('upper', 'strtoupper');

        $result = $this->js->eval("['hello', 'world'].map(x => upper(x)).join(' ')");
        $this->assertSame('HELLO WORLD', $result);
    }

    public function testRegisterInvalidFunctionThrows(): void
    {
        $this->expectException(\Exception::class);
        $this->js->registerFunction('invalid', 'nonexistent_php_function');
    }

    public function testFunctionReturnsFloat(): void
    {
        $this->js->registerFunction('sqrt', 'sqrt');

        $result = $this->js->eval("sqrt(16)");
        $this->assertSame(4, $result);
    }

    public function testFunctionReturnsArray(): void
    {
        $this->js->registerFunction('reverse', 'array_reverse');

        $result = $this->js->eval("reverse([1, 2, 3])");
        $this->assertSame([3, 2, 1], $result);
    }

    public function testChainMultipleFunctions(): void
    {
        $this->js->registerFunction('upper', 'strtoupper');
        $this->js->registerFunction('trim', 'trim');

        $result = $this->js->eval("upper(trim('  hello  '))");
        $this->assertSame('HELLO', $result);
    }

    public function testCanCreateQuickObject(): void
    {
        $obj = new QuickObject();
        $this->assertInstanceOf(QuickObject::class, $obj);
    }

    public function testQuickObjectWithProperties(): void
    {
        $obj = new QuickObject();
        $obj->registerProperty('version', '1.0.0');
        $obj->registerProperty('debug', true);

        $this->js->registerObject('Config', $obj);

        $this->assertSame('1.0.0', $this->js->eval("Config.version"));
        $this->assertTrue($this->js->eval("Config.debug"));
    }

    public function testQuickObjectWithMethods(): void
    {
        $obj = new QuickObject();
        $obj->registerFunction('upper', 'strtoupper');
        $obj->registerFunction('lower', 'strtolower');

        $this->js->registerObject('StringUtils', $obj);

        $this->assertSame('HELLO', $this->js->eval("StringUtils.upper('hello')"));
        $this->assertSame('hello', $this->js->eval("StringUtils.lower('HELLO')"));
    }

    public function testQuickObjectWithPropertiesAndMethods(): void
    {
        $obj = new QuickObject();
        $obj->registerProperty('version', PHP_VERSION);
        $obj->registerProperty('os', PHP_OS);
        $obj->registerFunction('strlen', 'strlen');
        $obj->registerFunction('md5', 'md5');

        $this->js->registerObject('PHP', $obj);

        $this->assertSame(PHP_VERSION, $this->js->eval("PHP.version"));
        $this->assertSame(PHP_OS, $this->js->eval("PHP.os"));
        $this->assertSame(5, $this->js->eval("PHP.strlen('hello')"));
        $this->assertSame(md5('test'), $this->js->eval("PHP.md5('test')"));
    }

    public function testQuickObjectMathLike(): void
    {
        $obj = new QuickObject();
        $obj->registerProperty('PI', M_PI);
        $obj->registerProperty('E', M_E);
        $obj->registerFunction('sqrt', 'sqrt');
        $obj->registerFunction('pow', 'pow');
        $obj->registerFunction('abs', 'abs');
        $obj->registerFunction('round', 'round');

        $this->js->registerObject('PHPMath', $obj);

        $this->assertEqualsWithDelta(M_PI, $this->js->eval("PHPMath.PI"), 0.00001);
        $this->assertSame(4, $this->js->eval("PHPMath.sqrt(16)"));
        $this->assertSame(8, $this->js->eval("PHPMath.pow(2, 3)"));
        $this->assertSame(5, $this->js->eval("PHPMath.abs(-5)"));
        $this->assertSame(4, $this->js->eval("PHPMath.round(3.7)"));
    }

    public function testQuickObjectWithNestedObject(): void
    {
        $config = new QuickObject();
        $config->registerProperty('host', 'localhost');
        $config->registerProperty('port', 3306);

        $app = new QuickObject();
        $app->registerProperty('name', 'MyApp');
        $app->registerObject('database', $config);

        $this->js->registerObject('App', $app);

        $this->assertSame('MyApp', $this->js->eval("App.name"));
        $this->assertSame('localhost', $this->js->eval("App.database.host"));
        $this->assertSame(3306, $this->js->eval("App.database.port"));
    }

    public function testQuickObjectWithDeeplyNestedObjects(): void
    {
        $level3 = new QuickObject();
        $level3->registerProperty('value', 'deep');

        $level2 = new QuickObject();
        $level2->registerObject('level3', $level3);

        $level1 = new QuickObject();
        $level1->registerObject('level2', $level2);

        $root = new QuickObject();
        $root->registerObject('level1', $level1);

        $this->js->registerObject('Root', $root);

        $this->assertSame('deep', $this->js->eval("Root.level1.level2.level3.value"));
    }

    public function testQuickObjectWithNestedObjectAndMethods(): void
    {
        $utils = new QuickObject();
        $utils->registerFunction('upper', 'strtoupper');
        $utils->registerFunction('lower', 'strtolower');

        $app = new QuickObject();
        $app->registerProperty('version', '1.0.0');
        $app->registerObject('utils', $utils);

        $this->js->registerObject('App', $app);

        $this->assertSame('1.0.0', $this->js->eval("App.version"));
        $this->assertSame('HELLO', $this->js->eval("App.utils.upper('hello')"));
        $this->assertSame('hello', $this->js->eval("App.utils.lower('HELLO')"));
    }

    public function testQuickObjectWithArrayProperty(): void
    {
        $obj = new QuickObject();
        $obj->registerProperty('items', ['a', 'b', 'c']);

        $this->js->registerObject('Config', $obj);

        $result = $this->js->eval("Config.items.join('-')");
        $this->assertSame('a-b-c', $result);
    }

    public function testQuickObjectWithAssociativeArrayProperty(): void
    {
        $obj = new QuickObject();
        $obj->registerProperty('database', [
            'host' => 'localhost',
            'port' => 3306,
        ]);

        $this->js->registerObject('Config', $obj);

        $this->assertSame('localhost', $this->js->eval("Config.database.host"));
        $this->assertSame(3306, $this->js->eval("Config.database.port"));
    }

    public function testQuickObjectInvalidFunctionThrows(): void
    {
        $this->expectException(\Exception::class);

        $obj = new QuickObject();
        $obj->registerFunction('broken', 'nonexistent_function');
    }

    public function testQuickObjectInComplexExpression(): void
    {
        $obj = new QuickObject();
        $obj->registerFunction('trim', 'trim');
        $obj->registerFunction('upper', 'strtoupper');
        $obj->registerFunction('len', 'strlen');

        $this->js->registerObject('PHP', $obj);

        $result = $this->js->eval("
            const text = '  Hello World  ';
            ({
                trimmed: PHP.trim(text),
                upper: PHP.upper(PHP.trim(text)),
                length: PHP.len(PHP.trim(text))
            })
        ");

        $this->assertSame('Hello World', $result['trimmed']);
        $this->assertSame('HELLO WORLD', $result['upper']);
        $this->assertSame(11, $result['length']);
    }

    public function testQuickObjectArrayUtils(): void
    {
        $obj = new QuickObject();
        $obj->registerFunction('sum', 'array_sum');
        $obj->registerFunction('count', 'count');
        $obj->registerFunction('reverse', 'array_reverse');
        $obj->registerFunction('implode', 'implode');

        $this->js->registerObject('ArrayUtils', $obj);

        $this->assertSame(15, $this->js->eval("ArrayUtils.sum([1, 2, 3, 4, 5])"));
        $this->assertSame(3, $this->js->eval("ArrayUtils.count([1, 2, 3])"));
        $this->assertSame([3, 2, 1], $this->js->eval("ArrayUtils.reverse([1, 2, 3])"));
        $this->assertSame('a-b-c', $this->js->eval("ArrayUtils.implode('-', ['a', 'b', 'c'])"));
    }

    public function testAddObjectPropertyCreatesObject(): void
    {
        $this->js->addObjectProperty('NewObj', 'prop1', 'value1');

        $this->assertSame('value1', $this->js->eval("NewObj.prop1"));
    }

    public function testAddObjectPropertyToExistingObject(): void
    {
        $this->js->addObjectProperty('Obj', 'prop1', 'value1');
        $this->js->addObjectProperty('Obj', 'prop2', 'value2');

        $this->assertSame('value1', $this->js->eval("Obj.prop1"));
        $this->assertSame('value2', $this->js->eval("Obj.prop2"));
    }

    public function testAddObjectMethodCreatesObject(): void
    {
        $this->js->addObjectMethod('Utils', 'upper', 'strtoupper');

        $this->assertSame('HELLO', $this->js->eval("Utils.upper('hello')"));
    }

    public function testAddObjectMethodToExistingObject(): void
    {
        $this->js->addObjectMethod('Utils', 'upper', 'strtoupper');
        $this->js->addObjectMethod('Utils', 'lower', 'strtolower');

        $this->assertSame('HELLO', $this->js->eval("Utils.upper('hello')"));
        $this->assertSame('hello', $this->js->eval("Utils.lower('HELLO')"));
    }

    public function testMixPropertiesAndMethodsIncrementally(): void
    {
        $this->js->addObjectProperty('App', 'name', 'MyApp');
        $this->js->addObjectProperty('App', 'version', '2.0.0');
        $this->js->addObjectMethod('App', 'hash', 'md5');

        $this->assertSame('MyApp', $this->js->eval("App.name"));
        $this->assertSame('2.0.0', $this->js->eval("App.version"));
        $this->assertSame(md5('test'), $this->js->eval("App.hash('test')"));
    }

    public function testAddMethodWithInvalidCallableThrows(): void
    {
        $this->expectException(\Exception::class);
        $this->js->addObjectMethod('Obj', 'broken', 'nonexistent_function');
    }
}
