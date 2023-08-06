use wasm_minimal_protocol::*;

initiate_protocol!();

#[wasm_func]
pub fn hello() -> Vec<u8> {
    b"Hello from wasm!!!".to_vec()
}

#[wasm_func]
pub fn double_it(arg: &[u8]) -> Vec<u8> {
    [arg, arg].concat()
}

#[wasm_func]
pub fn concatenate(arg1: &[u8], arg2: &[u8]) -> Vec<u8> {
    [arg1, b"*", arg2].concat()
}

#[wasm_func]
pub fn shuffle(arg1: &[u8], arg2: &[u8], arg3: &[u8]) -> Vec<u8> {
    [arg3, b"-", arg1, b"-", arg2].concat()
}

#[wasm_func]
pub fn returns_ok() -> Result<Vec<u8>, String> {
    Ok(b"This is an `Ok`".to_vec())
}

#[wasm_func]
pub fn returns_err() -> Result<Vec<u8>, String> {
    Err(String::from("This is an `Err`"))
}

#[wasm_func]
pub fn will_panic() -> Vec<u8> {
    panic!("unconditional panic")
}
