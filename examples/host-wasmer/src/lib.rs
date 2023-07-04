use wasmer::{
    Function, FunctionEnv, FunctionEnvMut, Imports, Instance, Memory, MemoryType, Module, Store,
    Value,
};

#[derive(Debug)]
struct PersistentData {
    memory: Memory,
    result_data: String,
    arg_buffer: String,
}

#[derive(Debug)]
pub struct PluginInstance {
    pub store: Store,
    memory: Memory,
    functions: Vec<(String, wasmer::Function)>,
    persistent_data: FunctionEnv<PersistentData>,
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
        self.functions == other.functions
    }
}

impl PluginInstance {
    pub fn new_from_bytes(bytes: impl AsRef<[u8]>) -> Result<Self, String> {
        let mut store = Store::default();
        let module =
            Module::new(&store, bytes).map_err(|err| format!("Couldn't load module: {err}"))?;

        let dummy_memory = Memory::new(&mut store, MemoryType::new(0, None, false)).unwrap();
        let persistent_data = FunctionEnv::new(
            &mut store,
            PersistentData {
                memory: dummy_memory,
                result_data: String::new(),
                arg_buffer: String::new(),
            },
        );
        let mut import_object = Imports::new();
        import_object.define(
            "typst_env",
            "wasm_minimal_protocol_send_result_to_host",
            Function::new_typed_with_env(
                &mut store,
                &persistent_data,
                |mut env: FunctionEnvMut<PersistentData>, ptr: u32, len: u32| {
                    let (data, store) = env.data_and_store_mut();
                    let mut buffer = vec![0u8; len as usize];
                    data.memory
                        .view(&store)
                        .read(ptr as u64, &mut buffer)
                        .unwrap();
                    data.result_data = String::from_utf8(buffer).unwrap();
                },
            ),
        );
        import_object.define(
            "typst_env",
            "wasm_minimal_protocol_write_args_to_buffer",
            Function::new_typed_with_env(
                &mut store,
                &persistent_data,
                |mut env: FunctionEnvMut<PersistentData>, ptr: u32| {
                    let (data, store) = env.data_and_store_mut();
                    data.memory
                        .view(&store)
                        .write(ptr as u64, data.arg_buffer.as_bytes())
                        .unwrap();
                },
            ),
        );

        let instance = Instance::new(&mut store, &module, &import_object)
            .map_err(|err| format!("Couldn't create a wasm instance: {err}"))?;

        // important functions that we will often use NOT AHAH 🤣

        let memory = instance.exports.get_memory("memory").unwrap().clone();
        persistent_data.as_mut(&mut store).memory = memory.clone();

        let functions = instance
            .exports
            .iter()
            .filter_map(|(s, e)| match e {
                wasmer::Extern::Function(f) => Some((s.to_owned(), f.clone())),
                _ => None,
            })
            .collect::<Vec<_>>();

        Ok(Self {
            store,
            memory,
            persistent_data,
            functions,
        })
    }

    /// Write arguments in `__RESULT`.
    pub fn write(&mut self, args: &[&str]) {
        let mut all_args = String::new();
        for arg in args {
            all_args += arg;
        }
        self.persistent_data.as_mut(&mut self.store).arg_buffer = all_args;
    }

    pub fn call(&mut self, function: &str, args: &[&str]) -> Result<String, String> {
        self.write(args);

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

        let s = std::mem::take(&mut self.persistent_data.as_mut(&mut self.store).result_data);

        if code != &Value::I32(0) {
            Err(format!(
                "plugin errored with: {:?} with code: {}",
                s,
                code.i32().unwrap()
            ))
        } else {
            Ok(s)
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

    pub fn iter_functions(&self) -> impl Iterator<Item = &String> {
        self.functions.as_slice().iter().map(|(x, _)| x)
    }
}
