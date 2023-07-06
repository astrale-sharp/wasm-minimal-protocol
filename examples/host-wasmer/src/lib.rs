use wasmer::{
    AsStoreMut as _, AsStoreRef as _, Extern, Function, FunctionEnv, FunctionEnvMut, Instance,
    Memory, MemoryType, Module, Store, Value,
};
use wasmer_wasix::{generate_import_object_from_env, WasiEnv, WasiFunctionEnv};

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
    functions: Vec<(String, Function)>,
    persistent_data: FunctionEnv<PersistentData>,
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
        let wasi_env = WasiEnv::builder("test").build().unwrap();
        let mut wasi_function_env = WasiFunctionEnv::new(&mut store, wasi_env);

        let mut import_object = generate_import_object_from_env(
            &mut store,
            &wasi_function_env.env,
            wasmer_wasix::WasiVersion::Latest,
        );
        import_object.define(
            "typst_env",
            "wasm_minimal_protocol_send_result_to_host",
            Function::new_typed_with_env(
                &mut store,
                &persistent_data,
                |mut env: FunctionEnvMut<PersistentData>, ptr: u32, len: u32| {
                    let memory = env.data().memory.clone();
                    let store = env.as_store_mut();
                    let mut buffer = vec![0u8; len as usize];
                    memory.view(&store).read(ptr as u64, &mut buffer).unwrap();
                    env.data_mut().result_data = String::from_utf8(buffer).unwrap();
                },
            ),
        );
        import_object.define(
            "typst_env",
            "wasm_minimal_protocol_write_args_to_buffer",
            Function::new_typed_with_env(
                &mut store,
                &persistent_data,
                |env: FunctionEnvMut<PersistentData>, ptr: u32| {
                    let data = env.data();
                    let store = env.as_store_ref();
                    data.memory
                        .view(&store)
                        .write(ptr as u64, data.arg_buffer.as_bytes())
                        .unwrap();
                },
            ),
        );

        let instance = Instance::new(&mut store, &module, &import_object)
            .map_err(|err| format!("Couldn't create a wasm instance: {err}"))?;

        let memory = instance.exports.get_memory("memory").unwrap().clone();
        persistent_data.as_mut(&mut store).memory = memory.clone();

        let functions = instance
            .exports
            .iter()
            .filter_map(|(s, e)| match e {
                Extern::Function(f) => Some((s.to_owned(), f.clone())),
                _ => None,
            })
            .collect::<Vec<_>>();
        wasi_function_env.initialize(&mut store, instance).unwrap();

        Ok(Self {
            store,
            memory,
            persistent_data,
            functions,
        })
    }

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
            .ok_or(format!("Plugin doesn't have the method: {function}"))?;

        let result_args = args
            .iter()
            .map(|a| Value::I32(a.len() as _))
            .collect::<Vec<_>>();

        let code = function
            .call(&mut self.store, &result_args)
            .map(|res| res.get(0).cloned().unwrap_or(Value::I32(3))) // if the function returns nothing
            .unwrap_or(Value::I32(2)); // in case of panic

        let s = std::mem::take(&mut self.persistent_data.as_mut(&mut self.store).result_data);

        match code {
            Value::I32(0) => Ok(s),
            Value::I32(1) => Err(format!(
                "plugin errored with: {:?} with code: {}",
                s,
                code.i32().unwrap()
            )),
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
        Some(function.clone())
    }

    pub fn iter_functions(&self) -> impl Iterator<Item = &String> {
        self.functions.as_slice().iter().map(|(x, _)| x)
    }
}
