use wasmi::{AsContext, Caller, Engine, Func as Function, Linker, Module, Value};

type Store = wasmi::Store<PersistentData>;

#[derive(Debug, Clone)]
struct PersistentData {
    result_data: Vec<u8>,
    arg_buffer: Vec<u8>,
}

#[derive(Debug)]
pub struct PluginInstance {
    store: Store,
    functions: Vec<(String, Function)>,
}

impl PluginInstance {
    pub fn new_from_bytes(bytes: impl AsRef<[u8]>) -> Result<Self, String> {
        let engine = Engine::default();
        let data = PersistentData {
            result_data: Vec::new(),
            arg_buffer: Vec::new(),
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
                    let memory = caller.get_export("memory").unwrap().into_memory().unwrap();
                    let mut buffer = std::mem::take(&mut caller.data_mut().result_data);
                    buffer.resize(len as usize, 0);
                    memory.read(&caller, ptr as _, &mut buffer).unwrap();
                    caller.data_mut().result_data = buffer;
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

        let functions = instance
            .exports(&store)
            .filter_map(|e| {
                let name = e.name().to_owned();
                e.into_func().map(|func| (name, func))
            })
            .collect::<Vec<_>>();
        Ok(Self { store, functions })
    }

    fn write(&mut self, args: &[&[u8]]) {
        self.store.data_mut().arg_buffer = args.concat();
    }

    pub fn call(&mut self, function: &str, args: &[&[u8]]) -> Result<Vec<u8>, String> {
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

        let s = std::mem::take(&mut self.store.data_mut().result_data);

        match code {
            Value::I32(0) => Ok(s),
            Value::I32(1) => Err(match String::from_utf8(s) {
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
