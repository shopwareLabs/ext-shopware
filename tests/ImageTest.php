<?php

declare(strict_types=1);

namespace Shopware\PHPExtension\Tests;

use PHPUnit\Framework\TestCase;
use Shopware\PHPExtension\Image\Image;

class ImageTest extends TestCase
{
    private string $testImagePath;
    private string $outputDir;

    protected function setUp(): void
    {
        $this->outputDir = sys_get_temp_dir() . '/php-ext-test-' . getmypid() . '/';
        $this->testImagePath = $this->outputDir . 'test_image.jpg';

        if (!is_dir($this->outputDir)) {
            mkdir($this->outputDir, 0755, true);
        }

        if (!file_exists($this->testImagePath)) {
            $this->createTestImage();
        }
    }

    private function createTestImage(): void
    {
        $image = imagecreatetruecolor(100, 100);
        $red = imagecolorallocate($image, 255, 0, 0);
        imagefill($image, 0, 0, $red);
        imagejpeg($image, $this->testImagePath);
        imagedestroy($image);
    }

    protected function tearDown(): void
    {
        $files = glob($this->outputDir . '*');
        foreach ($files as $file) {
            if (is_file($file)) {
                unlink($file);
            }
        }
    }

    public function testCanCreateInstanceFromFile(): void
    {
        $img = Image::fromFile($this->testImagePath);
        $this->assertInstanceOf(Image::class, $img);
    }

    public function testCanCreateInstanceFromString(): void
    {
        $data = file_get_contents($this->testImagePath);
        $img = Image::fromString($data);
        $this->assertInstanceOf(Image::class, $img);
    }

    public function testConstantsAreDefined(): void
    {
        $this->assertEquals('jpeg', Image::FORMAT_JPEG);
        $this->assertEquals('png', Image::FORMAT_PNG);
        $this->assertEquals('webp', Image::FORMAT_WEBP);
        $this->assertEquals('tiff', Image::FORMAT_TIFF);
        $this->assertEquals('jxl', Image::FORMAT_JXL);
        $this->assertEquals('avif', Image::FORMAT_AVIF);
    }

    public function testGetDimensionReturnsCorrectValues(): void
    {
        $img = Image::fromFile($this->testImagePath);
        $dimensions = $img->getDimension();

        $this->assertIsArray($dimensions);
        $this->assertArrayHasKey('width', $dimensions);
        $this->assertArrayHasKey('height', $dimensions);
        $this->assertEquals(100, $dimensions['width']);
        $this->assertEquals(100, $dimensions['height']);
    }

    public function testGetFormatReturnsLoader(): void
    {
        $img = Image::fromFile($this->testImagePath);
        $format = $img->getFormat();
        $this->assertIsString($format);
    }

    public function testSaveFileCreatesFile(): void
    {
        $img = Image::fromFile($this->testImagePath);
        $outputPath = $this->outputDir . 'saved.jpg';

        $img->saveFile($outputPath);
        $this->assertFileExists($outputPath);
    }

    public function testSaveFileWithFormatCreatesFile(): void
    {
        $img = Image::fromFile($this->testImagePath);
        $outputPath = $this->outputDir . 'saved_png.png';

        $img->saveFile($outputPath, Image::FORMAT_PNG);
        $this->assertFileExists($outputPath);

        $finfo = new \finfo(FILEINFO_MIME_TYPE);
        $mime = $finfo->file($outputPath);
        $this->assertEquals('image/png', $mime);
    }

    public function testSaveStringReturnsData(): void
    {
        $img = Image::fromFile($this->testImagePath);

        $data = $img->saveString(Image::FORMAT_PNG);
        $this->assertIsString($data);
        $this->assertNotEmpty($data);

        $finfo = new \finfo(FILEINFO_MIME_TYPE);
        $mime = $finfo->buffer($data);
        $this->assertEquals('image/png', $mime);
    }

    public function testResizeReturnsNewInstanceAndResizes(): void
    {
        $img = Image::fromFile($this->testImagePath);
        $outputPath = $this->outputDir . 'resized.jpg';

        $newImg = $img->resize(50, 50);

        $this->assertInstanceOf(Image::class, $newImg);
        $this->assertNotSame($img, $newImg);

        $dims = $newImg->getDimension();
        $this->assertEquals(50, $dims['width']);
        $this->assertEquals(50, $dims['height']);

        $origDims = $img->getDimension();
        $this->assertEquals(100, $origDims['width']);
        $this->assertEquals(100, $origDims['height']);

        $newImg->saveFile($outputPath);
        $this->assertFileExists($outputPath);
    }

    public function testCanSaveAsJxl(): void
    {
        $img = Image::fromFile($this->testImagePath);
        $outputPath = $this->outputDir . 'converted.jxl';

        $img->saveFile($outputPath, Image::FORMAT_JXL);
        $this->assertFileExists($outputPath);
    }

    public function testCanSaveAsAvif(): void
    {
        if (!Image::supportsFormat(Image::FORMAT_AVIF)) {
            $this->markTestSkipped('AVIF format is not supported');
        }

        $img = Image::fromFile($this->testImagePath);
        $outputPath = $this->outputDir . 'converted.avif';

        $img->saveFile($outputPath, Image::FORMAT_AVIF);
        $this->assertFileExists($outputPath);
    }

    public function testSaveFileWithQualityLowerSize(): void
    {
        $img = Image::fromFile($this->testImagePath);

        $pathHigh = $this->outputDir . 'high.jpg';
        $pathLow = $this->outputDir . 'low.jpg';

        $img->saveFile($pathHigh, Image::FORMAT_JPEG, 100);
        $img->saveFile($pathLow, Image::FORMAT_JPEG, 10);

        $this->assertFileExists($pathHigh);
        $this->assertFileExists($pathLow);

        $sizeHigh = filesize($pathHigh);
        $sizeLow = filesize($pathLow);

        $this->assertLessThan($sizeHigh, $sizeLow, 'Low quality image should be smaller than high quality image');
    }

    public function testSaveStringWithQualityLowerSize(): void
    {
        $img = Image::fromFile($this->testImagePath);

        $dataHigh = $img->saveString(Image::FORMAT_JPEG, 100);
        $dataLow = $img->saveString(Image::FORMAT_JPEG, 10);

        $this->assertLessThan(strlen($dataHigh), strlen($dataLow), 'Low quality string should be smaller than high quality string');
    }
}