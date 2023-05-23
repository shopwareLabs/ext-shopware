<?php

function random() {
    return md5(rand());
}

for ($i = 0; $i < 100; $i++) {
    $input = random();

    $encoded = \Shopware\Extension\zstd_encode($input);
    /** @var $decodedTestInstance TestObject */
    $decoded = \Shopware\Extension\zstd_decode($encoded);

    if ($input !== $decoded) {
        echo "Input: {$input} is not {$decoded}";
        exit(1);
    }
}
