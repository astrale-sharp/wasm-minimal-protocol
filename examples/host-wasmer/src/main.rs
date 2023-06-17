// this is an example of host
// you need to build the hello example first

use anyhow::Result;
use wasmer::{imports, Function, Instance, IntoBytes, Module, Store, Value};
fn main() -> Result<()> {
    let mut store = Store::default();
    let module = Module::new(&store, include_bytes!("../../hello.wasm"))?; // this is just compiled with the hello example
    let import_object = imports! {};
    let instance = Instance::new(&mut store, &module, &import_object)?;

    let read_at = instance.exports.get_function("read_at").unwrap();
    let hello_fn = instance.exports.get_function("hello").unwrap();
    let double_it = instance.exports.get_function("double_it").unwrap();
    let concatenate_fn = instance.exports.get_function("concatenate").unwrap();
    let shuffle = instance.exports.get_function("shuffle").unwrap();

    call(&instance, &mut store, hello_fn, vec![]);
    println!("{:?}", read(&instance, read_at, &mut store));

    call(&instance, &mut store, double_it, vec!["double me!!".into()]);
    println!("{:?}", read(&instance, read_at, &mut store));

    call(
        &instance,
        &mut store,
        concatenate_fn,
        vec![String::from("value1"), String::from("value2")],
    );
    println!("{:?}", read(&instance, read_at, &mut store));

    call(
        &instance,
        &mut store,
        shuffle,
        vec![
            String::from("value1"),
            String::from("value2"),
            String::from("value3"),
        ],
    );
    println!("{:?}", read(&instance, read_at, &mut store));

    Ok(())
}

fn write(instance: &Instance, store: &mut Store, to_write: impl IntoBytes) {
    instance
        .exports
        .get_function("clear")
        .unwrap()
        .call(store, &[])
        .unwrap();
    let push = instance.exports.get_function("push").unwrap();
    for b in to_write.into_bytes() {
        push.call(store, &[Value::I32(b as _)]).unwrap();
    }
}

fn read(instance: &Instance, read_at: &wasmer::Function, store: &mut Store) -> String {
    let len = instance
        .exports
        .get_function("get_len")
        .unwrap()
        .call(store, &[])
        .unwrap()[0]
        .i32()
        .unwrap();
    let mut res: Vec<u8> = vec![];
    for k in 0..len {
        let b = read_at.call(store, &[Value::I32(k)]).unwrap()[0]
            .i32()
            .unwrap();
        res.push(b as _)
    }
    String::from_utf8(res).expect("not a string")
}

fn call(instance: &Instance, store: &mut Store, function: &Function, params: Vec<String>) {
    let ty = function.ty(&store);
    // the protocol states the function takes argument to cut the result vec
    let mut p = vec![];
    if !ty.params().is_empty() {
        write(instance, store, params.join("").as_bytes());
        let params = {
            let mut v = vec![];
            let mut fold = 0;
            for k in params.iter() {
                fold += k.len();
                v.push(fold as i32);
            }
            v.pop();
            v
        };
        p = params
    };

    //clear
    let _ = instance
        .exports
        .get_function("clear")
        .unwrap()
        .call(store, &[])
        .unwrap();
    //
    write(instance, store, params.join("").as_bytes());
    let _ = function.call(
        store,
        p.into_iter().map(Value::I32).collect::<Vec<_>>().as_slice(),
    );
}
