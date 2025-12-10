#![cfg_attr(windows, feature(abi_vectorcall))]

use ext_php_rs::prelude::*;

mod quickjs;
mod lightningcss;
mod libvips;

pub use quickjs::{QuickJS, QuickObject};
pub use lightningcss::LightningCSS;
pub use libvips::Image;

#[php_module]
pub fn get_module(module: ModuleBuilder) -> ModuleBuilder {
    module
        .class::<QuickJS>()
        .class::<QuickObject>()
        .class::<LightningCSS>()
        .class::<Image>()
}
