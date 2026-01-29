#![cfg_attr(windows, feature(abi_vectorcall))]

use ext_php_rs::prelude::*;

mod quickjs;
mod lightningcss;
mod wasmtime_impl;

pub use quickjs::{QuickJS, QuickObject};
pub use lightningcss::LightningCSS;
pub use wasmtime_impl::{Wasmtime, WasmModule, WasmInstance};

#[php_module]
pub fn get_module(module: ModuleBuilder) -> ModuleBuilder {
    module
        .class::<QuickJS>()
        .class::<QuickObject>()
        .class::<LightningCSS>()
        .class::<Wasmtime>()
        .class::<WasmModule>()
        .class::<WasmInstance>()
}
