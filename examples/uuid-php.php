<?php

require __DIR__ . '/vendor/autoload.php';

for ($i = 0; $i < 10000; $i++) {
    \Ramsey\Uuid\Uuid::uuid7()->getBytes();
}
