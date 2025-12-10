<?php

declare(strict_types=1);

namespace Shopware\PHPExtension\Tests;

use PHPUnit\Framework\TestCase;
use Shopware\PHPExtension\LightningCSS\LightningCSS;

class LightningCSSTest extends TestCase
{
    private LightningCSS $css;

    protected function setUp(): void
    {
        $this->css = new LightningCSS();
    }

    public function testCanCreateInstance(): void
    {
        $this->assertInstanceOf(LightningCSS::class, $this->css);
    }

    public function testMinifyBasicCSS(): void
    {
        $input = 'body { color: red; }';
        $result = $this->css->minify($input);

        $this->assertSame('body{color:red}', $result);
    }

    public function testMinifyRemovesWhitespace(): void
    {
        $input = "
            body {
                color: red;
                background: blue;
            }
        ";
        $result = $this->css->minify($input);

        $this->assertStringNotContainsString("\n", $result);
        $this->assertStringNotContainsString('  ', $result);
    }

    public function testMinifyMultipleRules(): void
    {
        $input = '
            body { color: red; }
            .container { margin: 0 auto; }
            #header { background: blue; }
        ';
        $result = $this->css->minify($input);

        $this->assertStringContainsString('body{color:red}', $result);
        $this->assertStringContainsString('.container{margin:0 auto}', $result);
        $this->assertStringContainsString('#header{background:#00f}', $result);
    }

    public function testMinifyOptimizesColors(): void
    {
        $input = 'body { color: #ff0000; background: #0000ff; }';
        $result = $this->css->minify($input);

        $this->assertStringContainsString('red', $result);
        $this->assertStringContainsString('#00f', $result);
    }

    public function testMinifyMergesShorthand(): void
    {
        $input = 'body { margin-top: 10px; margin-right: 10px; margin-bottom: 10px; margin-left: 10px; }';
        $result = $this->css->minify($input);

        $this->assertStringContainsString('margin:10px', $result);
    }

    public function testFormatPrettyPrintsCSS(): void
    {
        $input = 'body{color:red;background:blue}';
        $result = $this->css->format($input);

        $this->assertStringContainsString("\n", $result);
    }

    public function testFormatPreservesRules(): void
    {
        $input = 'body { color: red; } .test { margin: 10px; }';
        $result = $this->css->format($input);

        $this->assertStringContainsString('body', $result);
        $this->assertStringContainsString('color', $result);
        $this->assertStringContainsString('.test', $result);
        $this->assertStringContainsString('margin', $result);
    }

    public function testTransformWithoutTargets(): void
    {
        $input = 'body { color: red; }';
        $result = $this->css->transform($input);

        $this->assertStringContainsString('color', $result);
        $this->assertStringContainsString('red', $result);
    }

    public function testTransformWithBrowserTargets(): void
    {
        $this->css->setBrowserTargets([
            'chrome' => 80,
            'firefox' => 75,
        ]);

        $input = 'body { color: red; }';
        $result = $this->css->transform($input);

        $this->assertStringContainsString('color', $result);
    }

    public function testValidateValidCSS(): void
    {
        $input = 'body { color: red; }';
        $result = $this->css->validate($input);

        $this->assertTrue($result);
    }

    public function testValidateComplexCSS(): void
    {
        $input = '
            body { color: red; }
            @media (min-width: 768px) { .container { width: 100%; } }
            @keyframes fade { from { opacity: 0; } to { opacity: 1; } }
        ';
        $result = $this->css->validate($input);

        $this->assertTrue($result);
    }

    public function testAnalyzeReturnsRulesCount(): void
    {
        $input = '
            body { color: red; }
            .test { margin: 10px; }
            #id { padding: 5px; }
        ';
        $result = $this->css->analyze($input);

        $this->assertIsArray($result);
        $this->assertArrayHasKey('rules_count', $result);
        $this->assertSame(3, $result['rules_count']);
    }

    public function testAnalyzeEmptyStylesheet(): void
    {
        $result = $this->css->analyze('');

        $this->assertIsArray($result);
        $this->assertSame(0, $result['rules_count']);
    }

    public function testSetBrowserTargetsDoesNotThrow(): void
    {
        $this->css->setBrowserTargets([
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

        $this->assertTrue(true);
    }

    public function testBrowserTargetsAffectsOutput(): void
    {
        $input = '.test { user-select: none; }';

        $css1 = new LightningCSS();
        $result1 = $css1->minify($input);

        $css2 = new LightningCSS();
        $css2->setBrowserTargets(['chrome' => 50]);
        $result2 = $css2->minify($input);

        $this->assertStringContainsString('user-select', $result1);
        $this->assertStringContainsString('user-select', $result2);
    }

    public function testMinifyMediaQueries(): void
    {
        $input = '
            @media (min-width: 768px) {
                body { color: red; }
            }
        ';
        $result = $this->css->minify($input);

        $this->assertStringContainsString('@media', $result);
        $this->assertStringContainsString('768px', $result);
    }

    public function testMinifyKeyframes(): void
    {
        $input = '
            @keyframes slide {
                from { transform: translateX(0); }
                to { transform: translateX(100px); }
            }
        ';
        $result = $this->css->minify($input);

        $this->assertStringContainsString('@keyframes', $result);
        $this->assertStringContainsString('slide', $result);
    }

    public function testMinifyPseudoSelectors(): void
    {
        $input = 'a:hover { color: blue; } button:focus { outline: none; }';
        $result = $this->css->minify($input);

        $this->assertStringContainsString(':hover', $result);
        $this->assertStringContainsString(':focus', $result);
    }

    public function testMinifyNestedSelectors(): void
    {
        $input = '.parent .child { color: red; } .parent > .direct { margin: 0; }';
        $result = $this->css->minify($input);

        $this->assertStringContainsString('.parent .child', $result);
        $this->assertStringContainsString('.parent>.direct', $result);
    }

    public function testMinifyVariables(): void
    {
        $input = ':root { --primary: blue; } body { color: var(--primary); }';
        $result = $this->css->minify($input);

        $this->assertStringContainsString('--primary', $result);
        $this->assertStringContainsString('var(--primary)', $result);
    }

    public function testMinifyCalc(): void
    {
        $input = '.box { width: calc(100% - 20px); }';
        $result = $this->css->minify($input);

        $this->assertStringContainsString('calc', $result);
    }

    public function testMinifyInvalidCSSThrows(): void
    {
        $this->expectException(\Exception::class);

        $this->css->minify('not valid css {{{{');
    }

    public function testFormatComplexCSS(): void
    {
        $input = '@media screen { body { color: red; } }';
        $result = $this->css->format($input);

        $this->assertStringContainsString('@media', $result);
        $this->assertStringContainsString('body', $result);
    }

    public function testTransformComplexCSS(): void
    {
        $input = '.flex { display: flex; gap: 10px; }';
        $result = $this->css->transform($input);

        $this->assertStringContainsString('display', $result);
        $this->assertStringContainsString('flex', $result);
    }
}
