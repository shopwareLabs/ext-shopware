use ext_php_rs::convert::IntoZvalDyn;
use ext_php_rs::prelude::*;
use ext_php_rs::types::{ArrayKey, ZendCallable, Zval};
use rquickjs::{Array, Context, Function as JsFunction, Object, Runtime, Value};
use std::cell::RefCell;
use std::sync::Arc;

/// Converts a PHP Zval to a QuickJS Value
fn zval_to_js<'js>(ctx: &rquickjs::Ctx<'js>, zval: &Zval) -> rquickjs::Result<Value<'js>> {
    if zval.is_null() {
        Ok(Value::new_null(ctx.clone()))
    } else if zval.is_bool() {
        let b = zval.bool().unwrap_or(false);
        Ok(Value::new_bool(ctx.clone(), b))
    } else if zval.is_long() {
        let n = zval.long().unwrap_or(0);
        Ok(Value::new_int(ctx.clone(), n as i32))
    } else if zval.is_double() {
        let n = zval.double().unwrap_or(0.0);
        Ok(Value::new_float(ctx.clone(), n))
    } else if zval.is_string() {
        let s = zval.string().unwrap_or_default();
        rquickjs::String::from_str(ctx.clone(), &s).map(|s| s.into())
    } else if zval.is_array() {
        let arr = zval.array().unwrap();

        // Check if it's a sequential array (list) or associative array (object)
        let mut is_sequential = true;
        let mut expected_index = 0i64;

        for (key, _) in arr.iter() {
            match key {
                ArrayKey::Long(idx) if idx == expected_index => {
                    expected_index += 1;
                }
                _ => {
                    is_sequential = false;
                    break;
                }
            }
        }

        if is_sequential {
            // Convert to JavaScript Array
            let js_arr = Array::new(ctx.clone())?;
            for (i, (_, val)) in arr.iter().enumerate() {
                let js_val = zval_to_js(ctx, val)?;
                js_arr.set(i, js_val)?;
            }
            Ok(js_arr.into())
        } else {
            // Convert to JavaScript Object
            let obj = Object::new(ctx.clone())?;
            for (key, val) in arr.iter() {
                let key_str = match key {
                    ArrayKey::Long(idx) => idx.to_string(),
                    ArrayKey::String(s) => s.to_string(),
                    ArrayKey::Str(s) => s.to_string(),
                };
                let js_val = zval_to_js(ctx, val)?;
                obj.set(&key_str, js_val)?;
            }
            Ok(obj.into())
        }
    } else {
        // Default to null for unsupported types
        Ok(Value::new_null(ctx.clone()))
    }
}

/// Converts a QuickJS Value to a PHP Zval
fn js_to_zval(value: &Value<'_>) -> Result<Zval, String> {
    let mut zval = Zval::new();

    if value.is_null() || value.is_undefined() {
        zval.set_null();
    } else if value.is_bool() {
        let b = value.as_bool().unwrap_or(false);
        zval.set_bool(b);
    } else if value.is_int() {
        let n = value.as_int().unwrap_or(0);
        zval.set_long(n as i64);
    } else if value.is_float() {
        let n = value.as_float().unwrap_or(0.0);
        zval.set_double(n);
    } else if let Some(s) = value.as_string() {
        let rust_str = s
            .to_string()
            .map_err(|e| format!("String conversion error: {:?}", e))?;
        zval.set_string(&rust_str, false)
            .map_err(|e| format!("Failed to set string: {:?}", e))?;
    } else if value.is_array() {
        let arr = value.as_array().unwrap();
        let mut php_arr = ext_php_rs::types::ZendHashTable::new();

        for i in 0..arr.len() {
            if let Ok(item) = arr.get::<Value>(i) {
                let item_zval = js_to_zval(&item)?;
                php_arr
                    .push(item_zval)
                    .map_err(|e| format!("Failed to push to array: {:?}", e))?;
            }
        }
        zval.set_hashtable(php_arr);
    } else if value.is_object() {
        let obj = value.as_object().unwrap();
        let mut php_arr = ext_php_rs::types::ZendHashTable::new();

        // Get all enumerable properties
        for key in obj.keys::<String>() {
            if let Ok(key) = key {
                if let Ok(val) = obj.get::<_, Value>(&key) {
                    let val_zval = js_to_zval(&val)?;
                    php_arr
                        .insert(key.as_str(), val_zval)
                        .map_err(|e| format!("Failed to insert to array: {:?}", e))?;
                }
            }
        }
        zval.set_hashtable(php_arr);
    } else if value.is_function() {
        // Functions are returned as a string representation
        zval.set_string("[Function]", false)
            .map_err(|e| format!("Failed to set string: {:?}", e))?;
    } else {
        zval.set_null();
    }

    Ok(zval)
}

/// Clone a Zval for storage - returns the cloned data as simple types
#[derive(Clone)]
enum StoredValue {
    Null,
    Bool(bool),
    Long(i64),
    Double(f64),
    String(String),
    Array(Vec<(String, StoredValue)>),
}

impl StoredValue {
    fn from_zval(val: &Zval) -> Self {
        if val.is_null() {
            StoredValue::Null
        } else if val.is_bool() {
            StoredValue::Bool(val.bool().unwrap_or(false))
        } else if val.is_long() {
            StoredValue::Long(val.long().unwrap_or(0))
        } else if val.is_double() {
            StoredValue::Double(val.double().unwrap_or(0.0))
        } else if val.is_string() {
            StoredValue::String(val.string().unwrap_or_default())
        } else if val.is_array() {
            if let Some(arr) = val.array() {
                let mut items = Vec::new();
                for (k, v) in arr.iter() {
                    let key = match k {
                        ArrayKey::Long(idx) => idx.to_string(),
                        ArrayKey::String(s) => s.to_string(),
                        ArrayKey::Str(s) => s.to_string(),
                    };
                    items.push((key, StoredValue::from_zval(v)));
                }
                StoredValue::Array(items)
            } else {
                StoredValue::Null
            }
        } else {
            StoredValue::Null
        }
    }

    fn to_js<'js>(&self, ctx: &rquickjs::Ctx<'js>) -> rquickjs::Result<Value<'js>> {
        match self {
            StoredValue::Null => Ok(Value::new_null(ctx.clone())),
            StoredValue::Bool(b) => Ok(Value::new_bool(ctx.clone(), *b)),
            StoredValue::Long(n) => Ok(Value::new_int(ctx.clone(), *n as i32)),
            StoredValue::Double(n) => Ok(Value::new_float(ctx.clone(), *n)),
            StoredValue::String(s) => {
                rquickjs::String::from_str(ctx.clone(), s).map(|s| s.into())
            }
            StoredValue::Array(items) => {
                // Check if it's sequential (all numeric keys starting from 0)
                let is_sequential = items.iter().enumerate().all(|(i, (k, _))| {
                    k.parse::<usize>().map(|n| n == i).unwrap_or(false)
                });

                if is_sequential && !items.is_empty() {
                    let arr = Array::new(ctx.clone())?;
                    for (i, (_, v)) in items.iter().enumerate() {
                        arr.set(i, v.to_js(ctx)?)?;
                    }
                    Ok(arr.into())
                } else {
                    let obj = Object::new(ctx.clone())?;
                    for (k, v) in items {
                        obj.set(k.as_str(), v.to_js(ctx)?)?;
                    }
                    Ok(obj.into())
                }
            }
        }
    }
}

/// Represents a property in a QuickObject
#[derive(Clone)]
enum QuickObjectMember {
    Property(StoredValue),
    Function(String),
    NestedObject(QuickObjectData),
}

/// Internal data structure for QuickObject
#[derive(Clone, Default)]
struct QuickObjectData {
    members: Vec<(String, QuickObjectMember)>,
}

impl QuickObjectData {
    fn new() -> Self {
        Self { members: Vec::new() }
    }

    fn add_property(&mut self, name: String, value: StoredValue) {
        // Remove existing member with same name if any
        self.members.retain(|(n, _)| n != &name);
        self.members.push((name, QuickObjectMember::Property(value)));
    }

    fn add_function(&mut self, name: String, php_func: String) {
        self.members.retain(|(n, _)| n != &name);
        self.members.push((name, QuickObjectMember::Function(php_func)));
    }

    fn add_object(&mut self, name: String, obj: QuickObjectData) {
        self.members.retain(|(n, _)| n != &name);
        self.members.push((name, QuickObjectMember::NestedObject(obj)));
    }
}

/// QuickObject - A builder class for creating JavaScript objects
/// 
/// Example usage:
/// ```php
/// $obj = new QuickObject();
/// $obj->registerProperty('version', '1.0.0');
/// $obj->registerFunction('upper', 'strtoupper');
/// 
/// $nested = new QuickObject();
/// $nested->registerProperty('host', 'localhost');
/// $obj->registerObject('config', $nested);
/// 
/// $js->registerObject('MyApp', $obj);
/// ```
#[php_class]
#[php(name = "Shopware\\PHPExtension\\QuickJS\\QuickObject")]
pub struct QuickObject {
    data: RefCell<QuickObjectData>,
}

#[php_impl]
impl QuickObject {
    /// Create a new QuickObject
    pub fn __construct() -> Self {
        QuickObject {
            data: RefCell::new(QuickObjectData::new()),
        }
    }

    /// Register a property with a value
    pub fn register_property(&self, name: &str, value: &Zval) -> PhpResult<()> {
        let stored = StoredValue::from_zval(value);
        self.data.borrow_mut().add_property(name.to_string(), stored);
        Ok(())
    }

    /// Register a PHP function as a method
    pub fn register_function(&self, name: &str, php_function_name: &str) -> PhpResult<()> {
        // Validate that the function exists
        ZendCallable::try_from_name(php_function_name)
            .map_err(|e| PhpException::default(format!("Invalid callable '{}': {:?}", php_function_name, e)))?;
        
        self.data.borrow_mut().add_function(name.to_string(), php_function_name.to_string());
        Ok(())
    }

    /// Register a nested QuickObject
    pub fn register_object(&self, name: &str, obj: &QuickObject) -> PhpResult<()> {
        let nested_data = obj.data.borrow().clone();
        self.data.borrow_mut().add_object(name.to_string(), nested_data);
        Ok(())
    }
}

impl QuickObject {
    /// Get the internal data for use by QuickJS (not exposed to PHP)
    fn get_data(&self) -> QuickObjectData {
        self.data.borrow().clone()
    }
}

/// Helper function to build a JS object from QuickObjectData
fn build_js_object<'js>(ctx: &rquickjs::Ctx<'js>, data: &QuickObjectData) -> rquickjs::Result<Object<'js>> {
    let obj = Object::new(ctx.clone())?;

    for (name, member) in &data.members {
        match member {
            QuickObjectMember::Property(value) => {
                let js_val = value.to_js(ctx)?;
                obj.set(name.as_str(), js_val)?;
            }
            QuickObjectMember::Function(func_name) => {
                let callback = PhpFunctionCallback {
                    func_name: func_name.clone(),
                };
                let func = rquickjs::Function::new(ctx.clone(), callback)?;
                obj.set(name.as_str(), func)?;
            }
            QuickObjectMember::NestedObject(nested_data) => {
                let nested_obj = build_js_object(ctx, nested_data)?;
                obj.set(name.as_str(), nested_obj)?;
            }
        }
    }

    Ok(obj)
}

/// QuickJS JavaScript engine class for PHP
/// 
/// This class provides a simple interface to execute JavaScript code from PHP.
/// It combines runtime and context management into a single, easy-to-use class.
#[php_class]
#[php(name = "Shopware\\PHPExtension\\QuickJS\\QuickJS")]
pub struct QuickJS {
    runtime: Arc<Runtime>,
    context: RefCell<Context>,
}

#[php_impl]
impl QuickJS {
    /// Create a new QuickJS instance
    pub fn __construct() -> PhpResult<Self> {
        let runtime = Arc::new(Runtime::new()
            .map_err(|e| PhpException::default(format!("Failed to create runtime: {:?}", e)))?);
        let context = Context::full(&runtime)
            .map_err(|e| PhpException::default(format!("Failed to create context: {:?}", e)))?;

        Ok(QuickJS {
            runtime,
            context: RefCell::new(context),
        })
    }

    /// Set memory limit in bytes
    pub fn set_memory_limit(&self, limit: i64) -> PhpResult<()> {
        self.runtime.set_memory_limit(limit as usize);
        Ok(())
    }

    /// Set max stack size in bytes
    pub fn set_max_stack_size(&self, size: i64) -> PhpResult<()> {
        self.runtime.set_max_stack_size(size as usize);
        Ok(())
    }

    /// Run garbage collection
    pub fn gc(&self) -> PhpResult<()> {
        self.runtime.run_gc();
        Ok(())
    }

    /// Get memory usage in bytes
    pub fn memory_usage(&self) -> i64 {
        let usage = self.runtime.memory_usage();
        usage.memory_used_size as i64
    }

    /// Evaluate JavaScript code and return the result
    pub fn eval(&self, code: &str) -> PhpResult<Zval> {
        let ctx = self.context.borrow();

        ctx.with(|ctx| {
            let result: Result<Value, _> = ctx.eval(code);

            match result {
                Ok(value) => js_to_zval(&value).map_err(|e| PhpException::default(e)),
                Err(e) => Err(PhpException::default(format!("JavaScript error: {:?}", e))),
            }
        })
    }

    /// Evaluate JavaScript code from a file
    pub fn eval_file(&self, filename: &str) -> PhpResult<Zval> {
        let code = std::fs::read_to_string(filename)
            .map_err(|e| PhpException::default(format!("Failed to read file: {:?}", e)))?;

        let ctx = self.context.borrow();

        ctx.with(|ctx| {
            let result: Result<Value, _> = ctx.eval(code);

            match result {
                Ok(value) => js_to_zval(&value).map_err(|e| PhpException::default(e)),
                Err(e) => Err(PhpException::default(format!("JavaScript error: {:?}", e))),
            }
        })
    }

    /// Set a global variable in the JavaScript context
    pub fn set_global(&self, name: &str, value: &Zval) -> PhpResult<()> {
        let ctx = self.context.borrow();

        ctx.with(|ctx| {
            let js_value = zval_to_js(&ctx, value)
                .map_err(|e| PhpException::default(format!("Failed to convert value: {:?}", e)))?;

            let globals = ctx.globals();
            globals
                .set(name, js_value)
                .map_err(|e| PhpException::default(format!("Failed to set global: {:?}", e)))?;

            Ok(())
        })
    }

    /// Get a global variable from the JavaScript context
    pub fn get_global(&self, name: &str) -> PhpResult<Zval> {
        let ctx = self.context.borrow();

        ctx.with(|ctx| {
            let globals = ctx.globals();
            let value: Value = globals
                .get(name)
                .map_err(|e| PhpException::default(format!("Failed to get global: {:?}", e)))?;

            js_to_zval(&value).map_err(|e| PhpException::default(e))
        })
    }

    /// Call a JavaScript function by name with arguments
    pub fn call(&self, function_name: &str, args: &ext_php_rs::types::ZendHashTable) -> PhpResult<Zval> {
        let ctx = self.context.borrow();

        ctx.with(|ctx| {
            let globals = ctx.globals();
            let func: JsFunction = globals
                .get(function_name)
                .map_err(|e| PhpException::default(format!("Function not found: {:?}", e)))?;

            // Convert PHP arguments to JS values
            let mut js_args: Vec<Value> = Vec::new();
            for (_, arg) in args.iter() {
                let js_val = zval_to_js(&ctx, arg)
                    .map_err(|e| PhpException::default(format!("Failed to convert argument: {:?}", e)))?;
                js_args.push(js_val);
            }

            // Call the function using Rest wrapper for variable args
            let result: Value = func
                .call((rquickjs::function::Rest(js_args),))
                .map_err(|e| PhpException::default(format!("Function call failed: {:?}", e)))?;

            js_to_zval(&result).map_err(|e| PhpException::default(e))
        })
    }

    /// Check if a global variable exists
    pub fn has_global(&self, name: &str) -> PhpResult<bool> {
        let ctx = self.context.borrow();

        ctx.with(|ctx| {
            let globals = ctx.globals();
            Ok(globals.contains_key(name).unwrap_or(false))
        })
    }

    /// Get the type of a global variable as a string
    pub fn typeof_global(&self, name: &str) -> PhpResult<String> {
        let ctx = self.context.borrow();

        ctx.with(|ctx| {
            let globals = ctx.globals();
            let value: Value = globals
                .get(name)
                .map_err(|e| PhpException::default(format!("Failed to get global: {:?}", e)))?;

            let type_str = if value.is_undefined() {
                "undefined"
            } else if value.is_null() {
                "null"
            } else if value.is_bool() {
                "boolean"
            } else if value.is_int() || value.is_float() {
                "number"
            } else if value.is_string() {
                "string"
            } else if value.is_array() {
                "array"
            } else if value.is_function() {
                "function"
            } else if value.is_object() {
                "object"
            } else {
                "unknown"
            };

            Ok(type_str.to_string())
        })
    }

    /// Register a PHP function (by name) as a JavaScript function
    /// Pass the function name as a string, e.g., "strtoupper", "array_sum"
    pub fn register_function(&self, js_name: &str, php_function_name: &str) -> PhpResult<()> {
        // Validate that the function exists
        ZendCallable::try_from_name(php_function_name)
            .map_err(|e| PhpException::default(format!("Invalid callable '{}': {:?}", php_function_name, e)))?;

        let php_func_name = php_function_name.to_string();
        let js_name_owned = js_name.to_string();
        let ctx = self.context.borrow();

        ctx.with(|ctx| {
            // Create a closure that captures the PHP function name
            let callback = PhpFunctionCallback {
                func_name: php_func_name.clone(),
            };

            // Create a Rust function that will be exposed to JS
            let func = rquickjs::Function::new(ctx.clone(), callback)
                .map_err(|e| PhpException::default(format!("Failed to create JS function: {:?}", e)))?;

            // Register the function as a global
            let globals = ctx.globals();
            globals
                .set(js_name_owned.as_str(), func)
                .map_err(|e| {
                    PhpException::default(format!("Failed to set global function: {:?}", e))
                })?;

            Ok(())
        })
    }

    /// Register a global JavaScript object using a QuickObject
    /// 
    /// Example:
    /// ```php
    /// $obj = new QuickObject();
    /// $obj->registerProperty('version', '1.0.0');
    /// $obj->registerFunction('upper', 'strtoupper');
    /// $js->registerObject('MyApp', $obj);
    /// ```
    #[php(name = "registerObject")]
    pub fn register_object_from_quick_object(&self, js_name: &str, obj: &QuickObject) -> PhpResult<()> {
        let js_name_owned = js_name.to_string();
        let data = obj.get_data();
        let ctx = self.context.borrow();

        ctx.with(|ctx| {
            let js_obj = build_js_object(&ctx, &data)
                .map_err(|e| PhpException::default(format!("Failed to build object: {:?}", e)))?;
            
            let globals = ctx.globals();
            globals
                .set(js_name_owned.as_str(), js_obj)
                .map_err(|e| PhpException::default(format!("Failed to set global object: {:?}", e)))?;
            
            Ok(())
        })
    }

    /// Create or get a global object and add a method to it
    /// This allows building objects incrementally
    pub fn add_object_method(&self, object_name: &str, method_name: &str, php_function_name: &str) -> PhpResult<()> {
        // Validate that the function exists
        ZendCallable::try_from_name(php_function_name)
            .map_err(|e| PhpException::default(format!("Invalid callable '{}': {:?}", php_function_name, e)))?;

        let php_func_name = php_function_name.to_string();
        let ctx = self.context.borrow();

        ctx.with(|ctx| {
            let globals = ctx.globals();

            // Get or create the object
            let obj: Object = if globals.contains_key(object_name).unwrap_or(false) {
                globals.get(object_name)
                    .map_err(|e| PhpException::default(format!("Failed to get object '{}': {:?}", object_name, e)))?
            } else {
                let new_obj = Object::new(ctx.clone())
                    .map_err(|e| PhpException::default(format!("Failed to create object: {:?}", e)))?;
                globals.set(object_name, new_obj.clone())
                    .map_err(|e| PhpException::default(format!("Failed to set object: {:?}", e)))?;
                new_obj
            };

            // Create and add the method
            let callback = PhpFunctionCallback {
                func_name: php_func_name,
            };
            let func = rquickjs::Function::new(ctx.clone(), callback)
                .map_err(|e| PhpException::default(format!("Failed to create method: {:?}", e)))?;
            obj.set(method_name, func)
                .map_err(|e| PhpException::default(format!("Failed to set method: {:?}", e)))?;

            Ok(())
        })
    }

    /// Add a property to an existing or new global object
    pub fn add_object_property(&self, object_name: &str, property_name: &str, value: &Zval) -> PhpResult<()> {
        let ctx = self.context.borrow();

        ctx.with(|ctx| {
            let globals = ctx.globals();

            // Get or create the object
            let obj: Object = if globals.contains_key(object_name).unwrap_or(false) {
                globals.get(object_name)
                    .map_err(|e| PhpException::default(format!("Failed to get object '{}': {:?}", object_name, e)))?
            } else {
                let new_obj = Object::new(ctx.clone())
                    .map_err(|e| PhpException::default(format!("Failed to create object: {:?}", e)))?;
                globals.set(object_name, new_obj.clone())
                    .map_err(|e| PhpException::default(format!("Failed to set object: {:?}", e)))?;
                new_obj
            };

            // Add the property
            let js_val = zval_to_js(&ctx, value)
                .map_err(|e| PhpException::default(format!("Failed to convert value: {:?}", e)))?;
            obj.set(property_name, js_val)
                .map_err(|e| PhpException::default(format!("Failed to set property: {:?}", e)))?;

            Ok(())
        })
    }
}

/// Struct to hold PHP function callback data
#[derive(Clone)]
struct PhpFunctionCallback {
    func_name: String,
}

impl<'js> rquickjs::function::IntoJsFunc<'js, (rquickjs::Ctx<'js>, rquickjs::function::Rest<Value<'js>>)> for PhpFunctionCallback {
    fn param_requirements() -> rquickjs::function::ParamRequirement {
        rquickjs::function::ParamRequirement::any()
    }

    fn call<'a>(&self, params: rquickjs::function::Params<'a, 'js>) -> rquickjs::Result<Value<'js>> {
        let ctx = params.ctx().clone();
        
        // Get all arguments
        let mut php_args: Vec<Zval> = Vec::new();
        for i in 0..params.len() {
            if let Some(arg) = params.arg(i) {
                if let Ok(val) = arg.get::<Value>() {
                    match js_to_zval(&val) {
                        Ok(zval) => php_args.push(zval),
                        Err(_) => return Err(rquickjs::Error::Unknown),
                    }
                }
            }
        }

        // Call the PHP function
        let callable = ZendCallable::try_from_name(&self.func_name)
            .map_err(|_| rquickjs::Error::Unknown)?;

        let arg_refs: Vec<&dyn IntoZvalDyn> = php_args.iter().map(|z| z as &dyn IntoZvalDyn).collect();

        let result = callable.try_call(arg_refs)
            .map_err(|_| rquickjs::Error::Unknown)?;

        // Convert result back to JS
        zval_to_js(&ctx, &result).map_err(|_| rquickjs::Error::Unknown)
    }
}
