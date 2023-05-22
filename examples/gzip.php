<?php

$text = file_get_contents('random.txt');

for ($i = 0; $i < 100; $i++) {
    $bla = gzencode($text, 9);
    echo strlen($bla) . PHP_EOL;
    $shit = gzdecode($bla);

    if ($text != $shit) {
        die("FUCK");
    }
}