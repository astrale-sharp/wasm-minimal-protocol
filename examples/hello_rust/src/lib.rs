use wasm_minimal_protocol::*;

// Only necessary when using cbor for passing arguments.
use ciborium::{de::from_reader, ser::into_writer};

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

#[derive(serde::Deserialize)]
struct ComplexDataArgs {
    x: i32,
    y: f64,
}

#[wasm_func]
pub fn complex_data(arg: &[u8]) -> Vec<u8> {
    let args: ComplexDataArgs = from_reader(arg).unwrap();
    let sum = args.x as f64 + args.y;
    let mut out = Vec::new();
    into_writer(&sum, &mut out).unwrap();
    out
}

#[wasm_func]
pub fn set_to_a(arg: &mut [u8]) -> Vec<u8> {
    for c in &mut *arg {
        *c = b'a';
    }
    arg.to_vec()
}

#[wasm_func]
pub fn set_to_a_reuse_buffer(arg: &mut [u8]) -> &[u8] {
    for c in &mut *arg {
        *c = b'a';
    }
    arg
}
