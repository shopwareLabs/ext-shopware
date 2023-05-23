<?php

// Generate 1k UUIDs and check if we crash
for ($i = 0; $i < 10000; $i++) {
    \Shopware\Extension\uuidv7();

    // TOOD: Validate UUIDs
}
