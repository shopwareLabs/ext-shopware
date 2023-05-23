use anyhow::Result;
use ext_php_rs::prelude::*;
use ext_php_rs::types::Zval;
use ext_php_rs::zend::ClassEntry;
use std::io::Read;
use std::io::Write;
use zstd::{stream::Encoder, Decoder};

#[php_function(name = "Shopware\\Extension\\zstd_encode")]
pub fn zstd_encode(data: &mut Zval) -> PhpResult<Zval> {
    if !data.is_string() {
        return Err(invalid_argument_error(1, "string"));
    }
    let input_bytes = data
        .binary_slice()
        .expect("could not retrieve bytes of argument 1");

    let val = zstd_encode_inner(input_bytes).map_err(|e| format!("Error encoding data: {}", e))?;
    Ok(val)
}

fn zstd_encode_inner(data: &[u8]) -> Result<Zval> {
    let mut encoder = Encoder::new(Vec::new(), 0)?;

    encoder.write_all(data)?;

    let mut val = Zval::new();
    val.set_binary(encoder.finish()?);

    Ok(val)
}

#[php_function(name = "Shopware\\Extension\\zstd_decode")]
pub fn zstd_decode(data: &mut Zval) -> PhpResult<Zval> {
    if !data.is_string() {
        return Err(invalid_argument_error(1, "string"));
    }
    let input_bytes = data
        .binary_slice()
        .expect("could not retrieve bytes of argument 1");

    let val = zstd_decode_inner(input_bytes).map_err(|e| format!("Error decoding data: {}", e))?;
    Ok(val)
}

fn zstd_decode_inner(data: &[u8]) -> Result<Zval> {
    let mut decoder = Decoder::new(data)?;
    let mut output = Vec::new();

    decoder.read_to_end(&mut output)?;

    let mut val = Zval::new();
    val.set_binary(output);

    Ok(val)
}

fn invalid_argument_error(pos: u8, kind: &str) -> PhpException {
    let class = ClassEntry::try_find("InvalidArgumentException").unwrap();
    PhpException::new(format!("Argument {} must be a {}", pos, kind), 0, class)
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
