use ext_php_rs::convert::IntoZvalDyn;
use ext_php_rs::prelude::*;
use ext_php_rs::types::{ZendCallable, Zval};
use std::cell::RefCell;
use wasmtime::*;

fn zval_to_wasm_val(val: &Zval, ty: ValType) -> Result<Val, String> {
    match ty {
        ValType::I32 => {
            if val.is_long() {
                Ok(Val::I32(val.long().unwrap_or(0) as i32))
            } else if val.is_double() {
                Ok(Val::I32(val.double().unwrap_or(0.0) as i32))
            } else if val.is_bool() {
                Ok(Val::I32(if val.bool().unwrap_or(false) { 1 } else { 0 }))
            } else {
                Err(format!("Cannot convert {:?} to i32", val.get_type()))
            }
        }
        ValType::I64 => {
            if val.is_long() {
                Ok(Val::I64(val.long().unwrap_or(0)))
            } else if val.is_double() {
                Ok(Val::I64(val.double().unwrap_or(0.0) as i64))
            } else if val.is_bool() {
                Ok(Val::I64(if val.bool().unwrap_or(false) { 1 } else { 0 }))
            } else {
                Err(format!("Cannot convert {:?} to i64", val.get_type()))
            }
        }
        ValType::F32 => {
            if val.is_double() {
                Ok(Val::F32((val.double().unwrap_or(0.0) as f32).to_bits()))
            } else if val.is_long() {
                Ok(Val::F32((val.long().unwrap_or(0) as f32).to_bits()))
            } else {
                Err(format!("Cannot convert {:?} to f32", val.get_type()))
            }
        }
        ValType::F64 => {
            if val.is_double() {
                Ok(Val::F64(val.double().unwrap_or(0.0).to_bits()))
            } else if val.is_long() {
                Ok(Val::F64((val.long().unwrap_or(0) as f64).to_bits()))
            } else {
                Err(format!("Cannot convert {:?} to f64", val.get_type()))
            }
        }
        ValType::V128 => Err("v128 type is not supported".to_string()),
        ValType::Ref(_) => Err("Reference types (externref/funcref) are not supported".to_string()),
    }
}

fn wasm_val_to_zval(val: Val) -> Result<Zval, String> {
    let mut zval = Zval::new();

    match val {
        Val::I32(n) => {
            zval.set_long(n as i64);
        }
        Val::I64(n) => {
            zval.set_long(n);
        }
        Val::F32(bits) => {
            zval.set_double(f32::from_bits(bits) as f64);
        }
        Val::F64(bits) => {
            zval.set_double(f64::from_bits(bits));
        }
        Val::V128(_) => {
            return Err("v128 type is not supported".to_string());
        }
        Val::FuncRef(_) => {
            return Err("funcref type is not supported".to_string());
        }
        Val::ExternRef(_) => {
            return Err("externref type is not supported".to_string());
        }
        Val::AnyRef(_) => {
            return Err("anyref type is not supported".to_string());
        }
        Val::ExnRef(_) => {
            return Err("exnref type is not supported".to_string());
        }
        Val::ContRef(_) => {
            return Err("contref type is not supported".to_string());
        }
    }

    Ok(zval)
}

#[derive(Clone)]
struct HostFunction {
    module: String,
    name: String,
    php_func: String,
}

struct WasmtimeState {
    #[allow(dead_code)]
    host_functions: Vec<HostFunction>,
}

/// Wasmtime - WebAssembly runtime for PHP
///
/// Example usage:
/// ```php
/// $wasm = new Wasmtime();
/// $module = $wasm->loadModule($wasmBytes);
/// $instance = $wasm->instantiate($module);
/// $result = $instance->call('add', [1, 2]);
/// ```
#[php_class]
#[php(name = "Shopware\\PHPExtension\\Wasmtime\\Wasmtime")]
pub struct Wasmtime {
    engine: RefCell<Engine>,
    fuel_enabled: RefCell<bool>,
    fuel_amount: RefCell<u64>,
    host_functions: RefCell<Vec<HostFunction>>,
}

fn create_wasmtime_config(consume_fuel: bool) -> Config {
    let mut config = Config::new();
    config.wasm_gc(true);
    config.wasm_function_references(true);
    config.wasm_exceptions(true);
    if consume_fuel {
        config.consume_fuel(true);
    }
    config
}

#[php_impl]
impl Wasmtime {
    pub fn __construct() -> PhpResult<Self> {
        let config = create_wasmtime_config(false);
        let engine = Engine::new(&config)
            .map_err(|e| PhpException::default(format!("Failed to create engine: {:?}", e)))?;

        Ok(Wasmtime {
            engine: RefCell::new(engine),
            fuel_enabled: RefCell::new(false),
            fuel_amount: RefCell::new(0),
            host_functions: RefCell::new(Vec::new()),
        })
    }

    pub fn set_fuel_enabled(&self, enabled: bool) -> PhpResult<()> {
        *self.fuel_enabled.borrow_mut() = enabled;
        let config = create_wasmtime_config(enabled);
        let engine = Engine::new(&config)
            .map_err(|e| PhpException::default(format!("Failed to create engine: {:?}", e)))?;
        *self.engine.borrow_mut() = engine;
        Ok(())
    }

    pub fn set_fuel(&self, amount: i64) -> PhpResult<()> {
        *self.fuel_amount.borrow_mut() = amount as u64;
        Ok(())
    }

    pub fn register_function(
        &self,
        module_name: &str,
        func_name: &str,
        php_function_name: &str,
    ) -> PhpResult<()> {
        ZendCallable::try_from_name(php_function_name).map_err(|e| {
            PhpException::default(format!("Invalid callable '{}': {:?}", php_function_name, e))
        })?;

        self.host_functions.borrow_mut().push(HostFunction {
            module: module_name.to_string(),
            name: func_name.to_string(),
            php_func: php_function_name.to_string(),
        });

        Ok(())
    }

    pub fn load_module(&self, bytes: &str) -> PhpResult<WasmModule> {
        let bytes = bytes.as_bytes();
        let engine = self.engine.borrow();
        let module = if bytes.starts_with(b"(") {
            let wat_str = std::str::from_utf8(bytes)
                .map_err(|e| PhpException::default(format!("Invalid WAT text: {:?}", e)))?;
            Module::new(&*engine, wat_str)
                .map_err(|e| PhpException::default(format!("Failed to compile WAT: {:?}", e)))?
        } else {
            Module::new(&*engine, bytes)
                .map_err(|e| PhpException::default(format!("Failed to load module: {:?}", e)))?
        };

        Ok(WasmModule { module })
    }

    pub fn load_module_from_file(&self, path: &str) -> PhpResult<WasmModule> {
        let engine = self.engine.borrow();
        let module = Module::from_file(&*engine, path).map_err(|e| {
            PhpException::default(format!("Failed to load module from file: {:?}", e))
        })?;

        Ok(WasmModule { module })
    }

    pub fn validate_module(bytes: &str) -> PhpResult<bool> {
        let bytes = bytes.as_bytes();
        let engine = Engine::default();

        if bytes.starts_with(b"(") {
            let wat_str = match std::str::from_utf8(bytes) {
                Ok(s) => s,
                Err(_) => return Ok(false),
            };
            match Module::new(&engine, wat_str) {
                Ok(_) => Ok(true),
                Err(_) => Ok(false),
            }
        } else {
            match Module::new(&engine, bytes) {
                Ok(_) => Ok(true),
                Err(_) => Ok(false),
            }
        }
    }

    pub fn instantiate(&self, module: &WasmModule) -> PhpResult<WasmInstance> {
        let fuel_enabled = *self.fuel_enabled.borrow();
        let fuel_amount = *self.fuel_amount.borrow();
        let engine = self.engine.borrow();

        let module_bytes = module
            .module
            .serialize()
            .map_err(|e| PhpException::default(format!("Failed to serialize module: {:?}", e)))?;

        let actual_module = unsafe {
            Module::deserialize(&*engine, &module_bytes).map_err(|e| {
                PhpException::default(format!("Failed to deserialize module: {:?}", e))
            })?
        };

        let mut store = Store::new(
            &*engine,
            WasmtimeState {
                host_functions: self.host_functions.borrow().clone(),
            },
        );

        if fuel_enabled && fuel_amount > 0 {
            store
                .set_fuel(fuel_amount)
                .map_err(|e| PhpException::default(format!("Failed to set fuel: {:?}", e)))?;
        }

        let mut linker = Linker::new(&*engine);

        for host_fn in self.host_functions.borrow().iter() {
            let php_func_name = host_fn.php_func.clone();
            let module_name = host_fn.module.clone();
            let func_name = host_fn.name.clone();

            linker
                .func_wrap(
                    &module_name,
                    &func_name,
                    move |_caller: Caller<'_, WasmtimeState>, arg: i32| -> i32 {
                        let callable = match ZendCallable::try_from_name(&php_func_name) {
                            Ok(c) => c,
                            Err(_) => return 0,
                        };

                        let mut arg_zval = Zval::new();
                        arg_zval.set_long(arg as i64);

                        let arg_refs: Vec<&dyn IntoZvalDyn> = vec![&arg_zval as &dyn IntoZvalDyn];
                        match callable.try_call(arg_refs) {
                            Ok(result) => result.long().unwrap_or(0) as i32,
                            Err(_) => 0,
                        }
                    },
                )
                .map_err(|e| {
                    PhpException::default(format!("Failed to register host function: {:?}", e))
                })?;
        }

        let instance = linker
            .instantiate(&mut store, &actual_module)
            .map_err(|e| PhpException::default(format!("Failed to instantiate module: {:?}", e)))?;

        Ok(WasmInstance {
            store: RefCell::new(store),
            instance,
        })
    }
}

#[php_class]
#[php(name = "Shopware\\PHPExtension\\Wasmtime\\WasmModule")]
pub struct WasmModule {
    module: Module,
}

#[php_impl]
impl WasmModule {
    pub fn get_exports(&self) -> PhpResult<Zval> {
        let mut arr = ext_php_rs::types::ZendHashTable::new();

        for export in self.module.exports() {
            let type_str = match export.ty() {
                ExternType::Func(_) => "function",
                ExternType::Global(_) => "global",
                ExternType::Table(_) => "table",
                ExternType::Memory(_) => "memory",
                ExternType::Tag(_) => "tag",
            };

            let mut export_arr = ext_php_rs::types::ZendHashTable::new();
            let _ = export_arr.insert("name", export.name().to_string());
            let _ = export_arr.insert("type", type_str.to_string());

            let mut export_zval = Zval::new();
            export_zval.set_hashtable(export_arr);
            let _ = arr.push(export_zval);
        }

        let mut zval = Zval::new();
        zval.set_hashtable(arr);
        Ok(zval)
    }
}

#[php_class]
#[php(name = "Shopware\\PHPExtension\\Wasmtime\\WasmInstance")]
pub struct WasmInstance {
    store: RefCell<Store<WasmtimeState>>,
    instance: Instance,
}

#[php_impl]
impl WasmInstance {
    pub fn call(&self, name: &str, args: &ext_php_rs::types::ZendHashTable) -> PhpResult<Zval> {
        let mut store = self.store.borrow_mut();

        let func = self
            .instance
            .get_func(&mut *store, name)
            .ok_or_else(|| PhpException::default(format!("Function '{}' not found", name)))?;

        let func_ty = func.ty(&*store);
        let param_types: Vec<ValType> = func_ty.params().collect();

        let mut wasm_args: Vec<Val> = Vec::new();
        for (i, (_, arg)) in args.iter().enumerate() {
            if i >= param_types.len() {
                return Err(PhpException::default(format!(
                    "Too many arguments: expected {}, got {}",
                    param_types.len(),
                    args.len()
                )));
            }
            let wasm_val = zval_to_wasm_val(arg, param_types[i].clone())
                .map_err(|e| PhpException::default(e))?;
            wasm_args.push(wasm_val);
        }

        let result_types: Vec<ValType> = func_ty.results().collect();
        let mut results: Vec<Val> = result_types
            .iter()
            .map(|ty| match ty {
                ValType::I32 => Val::I32(0),
                ValType::I64 => Val::I64(0),
                ValType::F32 => Val::F32(0),
                ValType::F64 => Val::F64(0),
                _ => Val::I32(0),
            })
            .collect();

        func.call(&mut *store, &wasm_args, &mut results)
            .map_err(|e| PhpException::default(format!("Function call failed: {:?}", e)))?;

        if results.is_empty() {
            let mut zval = Zval::new();
            zval.set_null();
            Ok(zval)
        } else if results.len() == 1 {
            wasm_val_to_zval(results.remove(0)).map_err(|e| PhpException::default(e))
        } else {
            let mut arr = ext_php_rs::types::ZendHashTable::new();
            for result in results {
                let zval = wasm_val_to_zval(result).map_err(|e| PhpException::default(e))?;
                let _ = arr.push(zval);
            }
            let mut zval = Zval::new();
            zval.set_hashtable(arr);
            Ok(zval)
        }
    }

    pub fn get_exports(&self) -> PhpResult<Zval> {
        let mut store = self.store.borrow_mut();
        let mut arr = ext_php_rs::types::ZendHashTable::new();

        let export_names: Vec<String> = self
            .instance
            .exports(&mut *store)
            .map(|export| export.name().to_string())
            .collect();

        for name in export_names {
            let type_str = if self.instance.get_func(&mut *store, &name).is_some() {
                "function"
            } else if self.instance.get_global(&mut *store, &name).is_some() {
                "global"
            } else if self.instance.get_table(&mut *store, &name).is_some() {
                "table"
            } else if self.instance.get_memory(&mut *store, &name).is_some() {
                "memory"
            } else {
                "unknown"
            };

            let mut export_arr = ext_php_rs::types::ZendHashTable::new();
            let _ = export_arr.insert("name", name);
            let _ = export_arr.insert("type", type_str.to_string());

            let mut export_zval = Zval::new();
            export_zval.set_hashtable(export_arr);
            let _ = arr.push(export_zval);
        }

        let mut zval = Zval::new();
        zval.set_hashtable(arr);
        Ok(zval)
    }

    pub fn get_global(&self, name: &str) -> PhpResult<Zval> {
        let mut store = self.store.borrow_mut();

        let global = self
            .instance
            .get_global(&mut *store, name)
            .ok_or_else(|| PhpException::default(format!("Global '{}' not found", name)))?;

        let val = global.get(&mut *store);
        wasm_val_to_zval(val).map_err(|e| PhpException::default(e))
    }

    pub fn set_global(&self, name: &str, value: &Zval) -> PhpResult<()> {
        let mut store = self.store.borrow_mut();

        let global = self
            .instance
            .get_global(&mut *store, name)
            .ok_or_else(|| PhpException::default(format!("Global '{}' not found", name)))?;

        let ty = global.ty(&*store);
        if ty.mutability() != Mutability::Var {
            return Err(PhpException::default(format!(
                "Global '{}' is not mutable",
                name
            )));
        }

        let wasm_val =
            zval_to_wasm_val(value, ty.content().clone()).map_err(|e| PhpException::default(e))?;

        global
            .set(&mut *store, wasm_val)
            .map_err(|e| PhpException::default(format!("Failed to set global: {:?}", e)))?;

        Ok(())
    }

    pub fn read_memory(&self, offset: i64, length: i64) -> PhpResult<Vec<u8>> {
        let mut store = self.store.borrow_mut();

        let memory = self
            .instance
            .get_memory(&mut *store, "memory")
            .ok_or_else(|| PhpException::default("Memory 'memory' not found".to_string()))?;

        let data = memory.data(&*store);
        let offset = offset as usize;
        let length = length as usize;

        if offset + length > data.len() {
            return Err(PhpException::default(format!(
                "Memory access out of bounds: offset={}, length={}, size={}",
                offset,
                length,
                data.len()
            )));
        }

        Ok(data[offset..offset + length].to_vec())
    }

    pub fn write_memory(&self, offset: i64, bytes: Vec<u8>) -> PhpResult<()> {
        let mut store = self.store.borrow_mut();

        let memory = self
            .instance
            .get_memory(&mut *store, "memory")
            .ok_or_else(|| PhpException::default("Memory 'memory' not found".to_string()))?;

        let offset = offset as usize;
        let data = memory.data_mut(&mut *store);

        if offset + bytes.len() > data.len() {
            return Err(PhpException::default(format!(
                "Memory write out of bounds: offset={}, length={}, size={}",
                offset,
                bytes.len(),
                data.len()
            )));
        }

        data[offset..offset + bytes.len()].copy_from_slice(&bytes);
        Ok(())
    }

    pub fn read_int32(&self, offset: i64) -> PhpResult<i64> {
        let bytes = self.read_memory(offset, 4)?;
        let arr: [u8; 4] = bytes
            .try_into()
            .map_err(|_| PhpException::default("Failed to convert bytes to i32".to_string()))?;
        Ok(i32::from_le_bytes(arr) as i64)
    }

    pub fn read_int64(&self, offset: i64) -> PhpResult<i64> {
        let bytes = self.read_memory(offset, 8)?;
        let arr: [u8; 8] = bytes
            .try_into()
            .map_err(|_| PhpException::default("Failed to convert bytes to i64".to_string()))?;
        Ok(i64::from_le_bytes(arr))
    }

    pub fn read_float32(&self, offset: i64) -> PhpResult<f64> {
        let bytes = self.read_memory(offset, 4)?;
        let arr: [u8; 4] = bytes
            .try_into()
            .map_err(|_| PhpException::default("Failed to convert bytes to f32".to_string()))?;
        Ok(f32::from_le_bytes(arr) as f64)
    }

    pub fn read_float64(&self, offset: i64) -> PhpResult<f64> {
        let bytes = self.read_memory(offset, 8)?;
        let arr: [u8; 8] = bytes
            .try_into()
            .map_err(|_| PhpException::default("Failed to convert bytes to f64".to_string()))?;
        Ok(f64::from_le_bytes(arr))
    }

    pub fn read_string(&self, offset: i64, length: i64) -> PhpResult<String> {
        let bytes = self.read_memory(offset, length)?;
        String::from_utf8(bytes)
            .map_err(|e| PhpException::default(format!("Invalid UTF-8 string: {:?}", e)))
    }

    pub fn write_int32(&self, offset: i64, value: i64) -> PhpResult<()> {
        let bytes = (value as i32).to_le_bytes().to_vec();
        self.write_memory(offset, bytes)
    }

    pub fn write_int64(&self, offset: i64, value: i64) -> PhpResult<()> {
        let bytes = value.to_le_bytes().to_vec();
        self.write_memory(offset, bytes)
    }

    pub fn write_float32(&self, offset: i64, value: f64) -> PhpResult<()> {
        let bytes = (value as f32).to_le_bytes().to_vec();
        self.write_memory(offset, bytes)
    }

    pub fn write_float64(&self, offset: i64, value: f64) -> PhpResult<()> {
        let bytes = value.to_le_bytes().to_vec();
        self.write_memory(offset, bytes)
    }

    pub fn write_string(&self, offset: i64, value: &str) -> PhpResult<()> {
        let bytes = value.as_bytes().to_vec();
        self.write_memory(offset, bytes)
    }

    pub fn memory_size(&self) -> PhpResult<i64> {
        let mut store = self.store.borrow_mut();

        let memory = self
            .instance
            .get_memory(&mut *store, "memory")
            .ok_or_else(|| PhpException::default("Memory 'memory' not found".to_string()))?;

        Ok(memory.data_size(&*store) as i64)
    }
}
