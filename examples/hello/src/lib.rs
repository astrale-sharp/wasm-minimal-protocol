use wasm_minimal_protocol::*;
declare_protocol!();

#[wasm_func]
pub fn hello() -> String {
    String::from("Hello from wasm!!!")
}

#[wasm_func]
pub fn double_it(arg: String) -> String {
    format!("{}{}", arg, arg)
}

#[wasm_func]
pub fn concatenate(arg1: String, arg2: String) -> String {
    format!("{}--{}", arg1, arg2)
}

#[wasm_func]
pub fn shuffle(arg1: String, arg2: String, arg3: String) -> String {
    format!("{}-{}-{}", arg3, arg1, arg2)
}
