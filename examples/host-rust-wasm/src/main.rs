use std::io::Write;

use rust_wasm::*;

fn main() {
    let mut bytes = include_bytes!("../../hello.wasm").as_slice();
    let mut f = std::fs::File::create("hello.wasm").unwrap();
    f.write_all(bytes).unwrap();
//    let f = std::fs::File::open("../../hello.wasm").unwrap();    

    let mut store = init_store();
    let module = decode_module(f).unwrap(); //this fails for now, I have to assume it's the library that's the problem
    let exp = module_exports(&module);
    //do your checks
    drop(exp);
    let instance = instantiate_module(&mut store, module, &[]).unwrap();
    let hello = get_export(&instance, "hello").unwrap();
    let read_at = get_export(&instance, "read_at").unwrap();
    let get_len = get_export(&instance, "get_len").unwrap();
    call(hello, &mut store);
    let values::Value::I32(len) = *call(get_len, &mut store).first().unwrap() else {panic!()};

    let mut res: Vec<u8> = Vec::with_capacity(256);
    for k in 0..len {
        let v = call_with(read_at, &mut store, vec![values::Value::I32(k)]);
        let values::Value::I32(v) =  *v.first().unwrap() else {panic!()};
        res.push(v as _)
    }
    let res = String::from_utf8(res).expect("not a string");
    println!("{res:?}");
}

fn call(func: ExternVal, store: &mut Store) -> Vec<values::Value> {
    if let ExternVal::Func(addr) = func {
        invoke_func(store, addr, vec![]).unwrap()
    } else {
        panic!()
    }
}

fn call_with(func: ExternVal, store: &mut Store, values: Vec<values::Value>) -> Vec<values::Value> {
    if let ExternVal::Func(addr) = func {
        invoke_func(store, addr, values).unwrap()
    } else {
        panic!()
    }
}
