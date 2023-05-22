<?php

$text = (string) file_get_contents('random.txt');


for ($i = 0; $i < 100; $i++) {
    $encoded = \Shopware\Extension\zstd_encode($text);
    echo strlen($encoded) . PHP_EOL;
    $decoded = \Shopware\Extension\zstd_decode($encoded);

    if ($decoded !== $text) {
        die('FUCK');
    }
}