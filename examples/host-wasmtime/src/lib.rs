use wasmtime::{Caller, Engine, Func, Linker, Module, Store, Val};
use wasmtime_wasi::{WasiCtx, WasiCtxBuilder};

struct PersistentData {
    wasi_ctx: WasiCtx,
    result_data: String,
    arg_buffer: String,
}

impl std::fmt::Debug for PersistentData {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PersistentData")
            .field("result_data", &self.result_data)
            .field("arg_buffer", &self.arg_buffer)
            .finish()
    }
}

#[derive(Debug)]
pub struct PluginInstance {
    store: Store<PersistentData>,
    functions: Vec<(String, Func)>,
}

impl PluginInstance {
    pub fn new_from_bytes(bytes: impl AsRef<[u8]>) -> Result<Self, String> {
        let engine = Engine::default();
        let mut linker = Linker::new(&engine);
        wasmtime_wasi::add_to_linker(&mut linker, |data: &mut PersistentData| &mut data.wasi_ctx)
            .unwrap();

        let wasi_ctx = WasiCtxBuilder::new().inherit_stdio().build();
        let module = Module::new(&engine, bytes).unwrap();

        let mut store = Store::new(
            &engine,
            PersistentData {
                result_data: String::new(),
                arg_buffer: String::new(),
                wasi_ctx,
            },
        );
        let instance = linker
            .func_wrap(
                "typst_env",
                "wasm_minimal_protocol_send_result_to_host",
                move |mut caller: Caller<PersistentData>, ptr: u32, len: u32| {
                    let memory = caller.get_export("memory").unwrap().into_memory().unwrap();
                    let mut buffer = vec![0u8; len as usize];
                    memory.read(&caller, ptr as _, &mut buffer).unwrap();
                    caller.data_mut().result_data = String::from_utf8(buffer).unwrap();
                },
            )
            .unwrap()
            .func_wrap(
                "typst_env",
                "wasm_minimal_protocol_write_args_to_buffer",
                move |mut caller: Caller<PersistentData>, ptr: u32| {
                    let memory = caller.get_export("memory").unwrap().into_memory().unwrap();
                    let buffer = std::mem::take(&mut caller.data_mut().arg_buffer);
                    memory
                        .write(&mut caller, ptr as _, buffer.as_bytes())
                        .unwrap();
                    caller.data_mut().arg_buffer = buffer;
                },
            )
            .unwrap()
            .module(&mut store, "", &module)
            .unwrap()
            .instantiate(&mut store, &module)
            .map_err(|err| format!("Couldn't create a wasm instance: {err}"))?;

        let functions = instance
            .exports(&mut store)
            .filter_map(|e| {
                let name = e.name().to_owned();
                e.into_func().map(|func| (name, func))
            })
            .collect::<Vec<_>>();

        Ok(Self {
            store,
            functions,
        })
    }

    pub fn write(&mut self, args: &[&str]) {
        let mut all_args = String::new();
        for arg in args {
            all_args += arg;
        }
        self.store.data_mut().arg_buffer = all_args;
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
            .map(|a| Val::I32(a.len() as _))
            .collect::<Vec<_>>();

        let mut code = [Val::I32(2)];
        let _ = function.call(&mut self.store, &result_args, &mut code);

        let s = std::mem::take(&mut self.store.data_mut().result_data);

        match code[0] {
            Val::I32(0) => Ok(s),
            Val::I32(1) => Err(format!("plugin errored with: {:?}", s)),
            Val::I32(2) => Err("plugin panicked".to_string()),
            _ => Err("plugin did not respect the protocol".to_string()),
        }
    }

    pub fn has_function(&self, method: &str) -> bool {
        self.functions.iter().any(|(s, _)| s == method)
    }

    pub fn get_function(&self, function_name: &str) -> Option<Func> {
        let Some((_, function)) = self.functions.iter().find(|(s, _)| s == function_name) else {
            return None
        };
        Some(*function)
    }

    pub fn iter_functions(&self) -> impl Iterator<Item = &String> {
        self.functions.as_slice().iter().map(|(x, _)| x)
    }
}
