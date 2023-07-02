// this is an example of host
// you need to build the hello example first

use anyhow::Result;
use host_wasmer::PluginInstance;
use wasmer::{Instance, Module, Store};

fn main() -> Result<()> {
    let mut store = Store::default();
    let module = Module::new(
        &store,
        include_bytes!("../../../target/wasm32-unknown-unknown/debug/hello.wasm"),
    )?; // this is just compiled with the hello example
    let import_object = wasmer::imports! {};
    let instance = Instance::new(&mut store, &module, &import_object)?;

    let mut plugin_instance = PluginInstance::new(instance, store);

    println!("{:?}", plugin_instance.call("hello", &[]));
    println!("{:?}", plugin_instance.call("double_it", &["double me!!"]));
    println!(
        "{:?}",
        plugin_instance.call("concatenate", &["val1", "value2"])
    );
    println!(
        "{:?}",
        plugin_instance.call("shuffle", &["value1", "value2", "value3"])
    );

    Ok(())
}
