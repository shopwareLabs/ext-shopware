<?php

$text = (string) file_get_contents('random.txt');

for ($i = 0; $i < 100; $i++) {
    $bla = \Shopware\Extension\zstd_encode($text);
    \Shopware\Extension\zstd_decode($bla);
}