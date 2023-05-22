# PHP Extension for porting some PHP functions to Rust

## UUIDv7

`\Shopware\Extension\uuidv7()`

- Returns a UUIDv7 as binary, use `bin2hex` to convert to string

Runs 3,5x faster than userland UUID generator

## ZSTD

- `\Shopware\Extension\zstd_encode`
- `\Shopware\Extension\zstd_decode`

Allows to compress and decompress data using ZSTD. 20x faster than `gzcompress` and `gzuncompress`.