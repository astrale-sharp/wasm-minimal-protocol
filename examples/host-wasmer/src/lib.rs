use wasmer::{Instance, Memory, Store, Value};

#[derive(Debug)]
pub struct PluginInstance {
    store: Store,
    memory: Memory,
    allocate_storage: wasmer::Function,
    get_storage_pointer: wasmer::Function,
    get_storage_len: wasmer::Function,
    functions: Vec<(String, wasmer::Function)>,
}

impl std::hash::Hash for PluginInstance {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        let len = self.memory.view(&self.store).data_size();
        for k in 0..len {
            self.memory
                .view(&self.store)
                .read_u8(k as _)
                .unwrap()
                .hash(state);
        }
    }
}

impl PartialEq for PluginInstance {
    fn eq(&self, other: &Self) -> bool {
        self.allocate_storage == other.allocate_storage
            && self.get_storage_pointer == other.get_storage_pointer
            && self.get_storage_len == other.get_storage_len
            && self.functions == other.functions
    }
}

impl PluginInstance {
    pub fn new(instance: Instance, store: Store) -> Self {
        // important functions that we will often use.
        let allocate_storage = instance
            .exports
            .get_function("wasm_minimal_protocol_allocate_storage")
            .unwrap()
            .clone();
        let get_storage_pointer = instance
            .exports
            .get_function("wasm_minimal_protocol_get_storage_pointer")
            .unwrap()
            .clone();
        let get_storage_len = instance
            .exports
            .get_function("wasm_minimal_protocol_get_storage_len")
            .unwrap()
            .clone();

        let memory = instance.exports.get_memory("memory").unwrap().clone();

        let functions = instance
            .exports
            .iter()
            .filter_map(|(s, e)| match e {
                wasmer::Extern::Function(f) => Some((s.to_owned(), f.clone())),
                _ => None,
            })
            .collect::<Vec<_>>();
        // .map_err(|_| "")
        // .to_owned();

        Self {
            store,
            memory,
            allocate_storage,
            get_storage_pointer,
            get_storage_len,
            functions,
        }
    }

    /// Write arguments in `__RESULT`.
    pub fn write(&mut self, args: &[&str]) -> Result<(), String> {
        let total_len = args.iter().map(|a| a.len()).sum::<usize>();
        self.allocate_storage
            .call(&mut self.store, &[wasmer::Value::I32(total_len as _)])
            .unwrap();
        let mut storage_pointer =
            self.get_storage_pointer.call(&mut self.store, &[]).unwrap()[0].unwrap_i32() as u64;
        for arg in args {
            self.memory
                .view(&self.store)
                .write(storage_pointer, arg.as_bytes())
                .unwrap();
            storage_pointer += arg.len() as u64;
        }
        Ok(())
    }

    pub fn call(&mut self, function: &str, args: &[&str]) -> Result<String, String> {
        self.write(args)?;

        let (_, function) = self
            .functions
            .iter()
            .find(|(s, _)| s == function)
            .ok_or("Plugin doesn't have the method: {function}")?;

        let result_args = args
            .iter()
            .map(|a| wasmer::Value::I32(a.len() as _))
            .collect::<Vec<_>>();

        let code = &function.call(&mut self.store, &result_args).unwrap()[0];

        // Get the resulting string in `__RESULT`
        let storage_pointer =
            self.get_storage_pointer.call(&mut self.store, &[]).unwrap()[0].unwrap_i32() as u64;
        let len = self.get_storage_len.call(&mut self.store, &[]).unwrap()[0].unwrap_i32();

        let mut result = vec![0u8; len as usize];
        self.memory
            .view(&self.store)
            .read(storage_pointer, &mut result)
            .unwrap();

        let s = String::from_utf8(result).map_err(|_| "Plugin data is not utf8".into());

        if code != &Value::I32(0) {
            Err(format!(
                "plugin errored with: {:?} with code: {}",
                s,
                code.i32().unwrap()
            ))
        } else {
            s
        }
    }

    pub fn has_function(&self, method: &str) -> bool {
        self.functions.iter().any(|(s, _)| s == method)
    }

    pub fn get_function(&self, function_name: &str) -> Option<wasmer::Function> {
        let Some((_, function)) = self.functions.iter().find(|(s, _)| s == function_name) else {return None};
        Some(function.clone())
        //Some(function.clone())
    }
}
