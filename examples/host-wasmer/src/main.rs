// this is an example of host
// you need to build the hello example first

use anyhow::Result;
use host_wasmer::PluginInstance;
use std::process::Command;

fn main() -> Result<()> {
    let args: Vec<_> = std::env::args().skip(1).collect();
    if args.len() != 1 {
        anyhow::bail!("1 argument required: 'rust', 'zig' or 'c'")
    }
    let plugin_binary = match args[0].as_str() {
        "rust" => {
            println!("=== compiling the Rust plugin");
            Command::new("cargo")
                .arg("build")
                .arg("--target")
                .arg("wasm32-wasi")
                .current_dir("examples/hello_rust")
                .spawn()?
                .wait()?;
            println!("===");
            std::fs::read("./examples/hello_rust/target/wasm32-wasi/debug/hello.wasm")?
        }
        "zig" => {
            println!("=== compiling the Zig plugin");
            Command::new("zig")
                .arg("build-lib")
                .arg("hello.zig")
                .arg("-target")
                .arg("wasm32-freestanding")
                .arg("-dynamic")
                .arg("-rdynamic")
                .current_dir("examples/hello_zig")
                .spawn()?
                .wait()?;
            println!("===");
            std::fs::read("examples/hello_zig/hello.wasm")?
        }
        "c" => {
            println!("=== compiling the C plugin");
            Command::new("emcc")
                .arg("--no-entry")
                .arg("-s")
                .arg("ERROR_ON_UNDEFINED_SYMBOLS=0")
                .arg("-o")
                .arg("hello.wasm")
                .arg("hello.c")
                .current_dir("examples/hello_c")
                .spawn()?
                .wait()?;
            println!("===");
            std::fs::read("examples/hello_c/hello.wasm")?
        }
        _ => anyhow::bail!("unknown argument '{}'", args[0].as_str()),
    };

    let mut plugin_instance = PluginInstance::new_from_bytes(plugin_binary).unwrap();

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
