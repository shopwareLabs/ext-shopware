use ext_php_rs::prelude::*;
use ext_php_rs::types::Zval;
use std::io::Read;
use std::io::Write;
use zstd::{stream::Encoder, Decoder};

#[php_function(name = "Shopware\\Extension\\zstd_encode")]
pub fn zstd_encode(name: &mut Zval) -> Zval {
    let mut encoder = Encoder::new(Vec::new(), 0).unwrap();
    encoder.write_all(name.binary_slice().unwrap()).unwrap();
    let mut val = Zval::new();
    val.set_binary(encoder.finish().unwrap());
    val
}

#[php_function(name = "Shopware\\Extension\\zstd_decode")]
pub fn zstd_decode(data: &mut Zval) -> Zval {
    let mut decoder = Decoder::new(data.binary_slice().unwrap()).unwrap();
    let mut output = Vec::new();
    decoder.read_to_end(&mut output).unwrap();
    let mut val = Zval::new();
    val.set_binary(output);

    val
}

#[php_function(name = "Shopware\\Extension\\uuidv7")]
pub fn uuidv7() -> Zval {
    let uuid = {
        use std::time::{SystemTime, UNIX_EPOCH};

        let mut buf = rand::random::<u128>() & 0xFFF3FFFFFFFFFFFFFFF;

        // 48 bits unix timestamp in ms
        buf |= SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("SystemTime before UNIX Epoch")
            .as_millis()
            << 80;

        // version
        buf |= 0x7 << 76;

        // variant
        buf |= 0b10 << 62;

        uuid::Uuid::from_u128(buf)
    };

    let mut val = Zval::new();
    val.set_binary(uuid.into_bytes().into());
    val
}

#[php_module]
pub fn get_module(module: ModuleBuilder) -> ModuleBuilder {
    module
}
