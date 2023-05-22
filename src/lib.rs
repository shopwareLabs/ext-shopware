#![cfg_attr(windows, feature(abi_vectorcall))]
use std::collections::HashSet;

use anyhow::{anyhow, Result};
use ext_php_rs::prelude::*;
use ext_php_rs::types::{ZendHashTable, Zval};

#[php_module]
pub fn get_module(module: ModuleBuilder) -> ModuleBuilder {
    module
}
