use ext_php_rs::prelude::*;
use ext_php_rs::types::Zval;
use std::io::Read;
use std::io::Write;
use zstd::{stream::Encoder, Decoder};

#[php_function(name = "Shopware\\Extension\\zstd_encode")]
pub fn zstd_encode(name: &mut Zval) -> Zval {
    let mut buf: Vec<u8> = Vec::new();
    let mut encoder = Encoder::new(buf, 0).unwrap();
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

#[php_module]
pub fn get_module(module: ModuleBuilder) -> ModuleBuilder {
    module
}
