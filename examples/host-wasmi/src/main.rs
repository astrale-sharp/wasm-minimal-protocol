use wasmi::*;

fn main() {
    let engine = Engine::default();
    let module = Module::new(&engine, include_bytes!("../../hello.wasm").as_slice()).unwrap();
    let mut store = Store::new(&engine, 0u32);
    let linker = <Linker<u32>>::new(&engine);
    let instance = linker
        .instantiate(&mut store, &module)
        .unwrap()
        .start(&mut store)
        .unwrap();
    
    let hello = instance.get_func(&store, "hello").unwrap();
    let get_len = instance.get_func(&store, "get_len").unwrap();
    let read_at = instance.get_func(&store, "read_at").unwrap();
    hello.call(&mut store, &[], &mut []).unwrap();
    let mut v = vec![Value::I32(0)];
    get_len.call(&mut store, &[], &mut v).unwrap();
    let len = v.first().unwrap().i32().unwrap();

    let mut res: Vec<u8> = Vec::with_capacity(256);
    for k in 0..len {
        let mut b = vec![Value::I32(0)];
        read_at.call(&mut store, &[Value::I32(k)], &mut b).unwrap();
        let b = b.first().unwrap().i32().unwrap();
        res.push(b as _)
    }
    let res = String::from_utf8(res).expect("not a string");
    println!("{res:?}");
}
