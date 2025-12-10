<?php

use PhpBench\Benchmark\Metadata\Annotations\BeforeMethods;
use PhpBench\Benchmark\Metadata\Annotations\AfterMethods;
use PhpBench\Benchmark\Metadata\Annotations\Revs;
use PhpBench\Benchmark\Metadata\Annotations\Iterations;
use Shopware\PHPExtension\Image\Image;

/**
 * @BeforeMethods({"setUp"})
 * @AfterMethods({"tearDown"})
 */
class ImageBench
{
    private string $imagePath;
    private string $outputPath;

    public function setUp(): void
    {
        $this->imagePath = __DIR__ . '/test_bench_image.jpg';
        $this->outputPath = __DIR__ . '/output_bench.jpg';

        // Create a large 2000x2000 image for benchmarking
        if (!file_exists($this->imagePath)) {
            $image = imagecreatetruecolor(2000, 2000);
            // Fill with random noise to simulate real content
            for ($y = 0; $y < 2000; $y += 50) {
                for ($x = 0; $x < 2000; $x += 50) {
                    $color = imagecolorallocate($image, rand(0, 255), rand(0, 255), rand(0, 255));
                    imagefilledrectangle($image, $x, $y, $x + 49, $y + 49, $color);
                }
            }
            imagejpeg($image, $this->imagePath, 90);
            imagedestroy($image);
        }
    }

    public function tearDown(): void
    {
        if (file_exists($this->outputPath)) {
            unlink($this->outputPath);
        }
    }

    /**
     * @Revs(5)
     * @Iterations(3)
     */
    public function benchVipsLoad(): void
    {
        Image::fromFile($this->imagePath);
    }

    /**
     * @Revs(5)
     * @Iterations(3)
     */
    public function benchGdLoad(): void
    {
        $img = imagecreatefromjpeg($this->imagePath);
        imagedestroy($img);
    }

    /**
     * @Revs(5)
     * @Iterations(3)
     */
    public function benchVipsResize(): void
    {
        $img = Image::fromFile($this->imagePath);
        $resized = $img->resize(800, 600);
    }

    /**
     * @Revs(5)
     * @Iterations(3)
     */
    public function benchGdResize(): void
    {
        $img = imagecreatefromjpeg($this->imagePath);
        $resized = imagescale($img, 800, 600);
        imagedestroy($img);
        imagedestroy($resized);
    }

    /**
     * @Revs(5)
     * @Iterations(3)
     */
    public function benchVipsSave(): void
    {
        $img = Image::fromFile($this->imagePath);
        $img->saveFile($this->outputPath, 'jpeg', 85);
    }

    /**
     * @Revs(5)
     * @Iterations(3)
     */
    public function benchGdSave(): void
    {
        $img = imagecreatefromjpeg($this->imagePath);
        imagejpeg($img, $this->outputPath, 85);
        imagedestroy($img);
    }

    /**
     * @Revs(5)
     * @Iterations(3)
     */
    public function benchVipsFullWorkflow(): void
    {
        $img = Image::fromFile($this->imagePath);
        $resized = $img->resize(800, 600);
        $resized->saveFile($this->outputPath, 'jpeg', 85);
    }

    /**
     * @Revs(5)
     * @Iterations(3)
     */
    public function benchGdFullWorkflow(): void
    {
        $img = imagecreatefromjpeg($this->imagePath);
        $resized = imagescale($img, 800, 600);
        imagejpeg($resized, $this->outputPath, 85);
        imagedestroy($img);
        imagedestroy($resized);
    }
}
