<?php

$text = file_get_contents('random.txt');

for ($i = 0; $i < 100; $i++) {
    $bla = gzencode($text, 9);
    gzdecode($bla);
}