// this is an example of host
// you need to build the hello example first

use anyhow::Result;
use host_wasmer::PluginInstance;

fn main() -> Result<()> {
    let mut plugin_instance = PluginInstance::new_from_bytes(include_bytes!(
        "../../../target/wasm32-unknown-unknown/debug/hello.wasm"
    ))
    .unwrap();

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
    println!("{:?}", plugin_instance.call("returns_ok", &[]));
    println!("{:?}", plugin_instance.call("returns_err", &[]));
    println!("{:?}", plugin_instance.call("will_panic", &[]));

    Ok(())
}
