use wasmi::{AsContext, Caller, Engine, Func as Function, Linker, Memory, Module, Value};

type Store = wasmi::Store<PersistentData>;

/// Reference to a slice of memory returned after
/// [calling a wasm function](PluginInstance::call).
///
/// # Drop
/// On [`Drop`], this will free the slice of memory inside the plugin.
///
/// As such, this structure mutably borrows the [`PluginInstance`], which prevents
/// another function from being called.
pub struct ReturnedData<'a> {
    memory: Memory,
    ptr: u32,
    len: u32,
    free_function: &'a Function,
    context_mut: &'a mut Store,
}

impl<'a> ReturnedData<'a> {
    /// Get a reference to the returned slice of data.
    ///
    /// # Panic
    /// This may panic if the function returned an invalid `(ptr, len)` pair.
    pub fn get(&self) -> &[u8] {
        &self.memory.data(&*self.context_mut)[self.ptr as usize..(self.ptr + self.len) as usize]
    }
}

impl Drop for ReturnedData<'_> {
    fn drop(&mut self) {
        self.free_function
            .call(
                &mut *self.context_mut,
                &[Value::I32(self.ptr as _), Value::I32(self.len as _)],
                &mut [],
            )
            .unwrap();
    }
}

#[derive(Debug, Clone)]
struct PersistentData {
    result_ptr: u32,
    result_len: u32,
    arg_buffer: Vec<u8>,
}

#[derive(Debug)]
pub struct PluginInstance {
    store: Store,
    memory: Memory,
    free_function: Function,
    functions: Vec<(String, Function)>,
}

impl PluginInstance {
    pub fn new_from_bytes(bytes: impl AsRef<[u8]>) -> Result<Self, String> {
        let engine = Engine::default();
        let data = PersistentData {
            arg_buffer: Vec::new(),
            result_ptr: 0,
            result_len: 0,
        };
        let mut store = Store::new(&engine, data);

        let module = Module::new(&engine, bytes.as_ref())
            .map_err(|err| format!("Couldn't load module: {err}"))?;

        let mut linker = Linker::new(&engine);
        let instance = linker
            .func_wrap(
                "typst_env",
                "wasm_minimal_protocol_send_result_to_host",
                move |mut caller: Caller<PersistentData>, ptr: u32, len: u32| {
                    caller.data_mut().result_ptr = ptr;
                    caller.data_mut().result_len = len;
                },
            )
            .unwrap()
            .func_wrap(
                "typst_env",
                "wasm_minimal_protocol_write_args_to_buffer",
                move |mut caller: Caller<PersistentData>, ptr: u32| {
                    let memory = caller.get_export("memory").unwrap().into_memory().unwrap();
                    let buffer = std::mem::take(&mut caller.data_mut().arg_buffer);
                    memory.write(&mut caller, ptr as _, &buffer).unwrap();
                    caller.data_mut().arg_buffer = buffer;
                },
            )
            .unwrap()
            .instantiate(&mut store, &module)
            .map_err(|e| format!("{e}"))?
            .start(&mut store)
            .map_err(|e| format!("{e}"))?;

        let mut free_function = None;
        let functions = instance
            .exports(&store)
            .filter_map(|e| {
                let name = e.name().to_owned();

                e.into_func().map(|func| {
                    if name == "wasm_minimal_protocol_free_byte_buffer" {
                        free_function = Some(func);
                    }
                    (name, func)
                })
            })
            .collect::<Vec<_>>();
        let free_function = free_function.unwrap();
        let memory = instance
            .get_export(&store, "memory")
            .unwrap()
            .into_memory()
            .unwrap();
        Ok(Self {
            store,
            memory,
            free_function,
            functions,
        })
    }

    fn write(&mut self, args: &[&[u8]]) {
        self.store.data_mut().arg_buffer = args.concat();
    }

    pub fn call(&mut self, function: &str, args: &[&[u8]]) -> Result<ReturnedData, String> {
        self.write(args);

        let (_, function) = self
            .functions
            .iter()
            .find(|(s, _)| s == function)
            .ok_or(format!("Plugin doesn't have the method: {function}"))?;

        let result_args = args
            .iter()
            .map(|a| Value::I32(a.len() as _))
            .collect::<Vec<_>>();

        let mut code = [Value::I32(2)];
        let is_err = function
            .call(&mut self.store, &result_args, &mut code)
            .is_err();
        let code = if is_err {
            Value::I32(2)
        } else {
            code.first().cloned().unwrap_or(Value::I32(3)) // if the function returns nothing
        };

        let (ptr, len) = (self.store.data().result_ptr, self.store.data().result_len);

        let result = ReturnedData {
            memory: self.memory,
            ptr,
            len,
            free_function: &self.free_function,
            context_mut: &mut self.store,
        };

        match code {
            Value::I32(0) => Ok(result),
            Value::I32(1) => Err(match std::str::from_utf8(result.get()) {
                Ok(err) => format!("plugin errored with: '{}'", err,),
                Err(_) => String::from("plugin errored and did not return valid UTF-8"),
            }),
            Value::I32(2) => Err("plugin panicked".to_string()),
            _ => Err("plugin did not respect the protocol".to_string()),
        }
    }

    pub fn has_function(&self, method: &str) -> bool {
        self.functions.iter().any(|(s, _)| s == method)
    }

    pub fn get_function(&self, function_name: &str) -> Option<Function> {
        let Some((_, function)) = self.functions.iter().find(|(s, _)| s == function_name) else {
            return None
        };
        Some(*function)
    }

    pub fn iter_functions(&self) -> impl Iterator<Item = &String> {
        self.functions.as_slice().iter().map(|(x, _)| x)
    }

    pub fn get_store(&self) -> &impl AsContext {
        &self.store
    }
}
