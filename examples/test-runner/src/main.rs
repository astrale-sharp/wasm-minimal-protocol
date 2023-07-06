// this is an example of host
// you need to build the hello example first

use anyhow::Result;
use std::process::Command;

#[cfg(feature = "host-wasmtime")]
use host_wasmtime::PluginInstance;

#[cfg(feature = "host-wasmer")]
use host_wasmer::PluginInstance;

#[cfg(feature = "host-wasmi")]
use host_wasmi::PluginInstance;

#[cfg(feature = "abi_unknown")]
mod consts {
    pub const RUST_TARGET: &str = "wasm32-unknown-unknown";
    pub const RUST_PATH: &str = "../hello_rust/target/wasm32-unknown-unknown/debug/hello.wasm";
    pub const ZIG_TARGET: &str = "wasm32-freestanding";
}

#[cfg(feature = "abi_wasi")]
mod consts {
    pub const RUST_TARGET: &str = "wasm32-wasi";
    pub const RUST_PATH: &str = "../hello_rust/target/wasm32-wasi/debug/hello.wasm";
    pub const ZIG_TARGET: &str = "wasm32-wasi";
}

#[cfg(any(
    all(feature = "host-wasmtime", feature = "host-wasmer"),
    all(feature = "host-wasmtime", feature = "host-wasmi"),
    all(feature = "host-wasmer", feature = "host-wasmi"),
))]
compile_error!("Only one feature in [host-wasmtime, host-wasmi, host-wasmer] must be specified.");

#[cfg(not(any(
    feature = "host-wasmtime",
    feature = "host-wasmer",
    feature = "host-wasmi"
)))]
compile_error!(
    "At least one feature in [host-wasmtime, host-wasmi, host-wasmer] must be specified."
);

fn main() -> Result<()> {
    let args: Vec<_> = std::env::args().skip(1).collect();
    if args.len() != 1 {
        anyhow::bail!("1 argument required: 'rust', 'zig' or 'c'")
    }
    let plugin_binary = match args[0].as_str() {
        #[cfg(any(feature = "abi_unknown", feature = "abi_wasi"))]
        "rust" => {
            println!("=== compiling the Rust plugin");
            Command::new("cargo")
                .arg("build")
                .arg("--target")
                .arg(consts::RUST_TARGET)
                .current_dir("../hello_rust")
                .spawn()?
                .wait()?;
            println!("===");
            std::fs::read(consts::RUST_PATH)?
        }
        #[cfg(any(feature = "abi_unknown", feature = "abi_wasi"))]
        "zig" => {
            println!("=== compiling the Zig plugin");
            Command::new("zig")
                .arg("build-lib")
                .arg("hello.zig")
                .arg("-target")
                .arg(consts::ZIG_TARGET)
                .arg("-dynamic")
                .arg("-rdynamic")
                .current_dir("../examples/hello_zig")
                .spawn()
                .expect("do you have zig installed and in the path?")
                .wait()?;
            println!("===");
            std::fs::read("../examples/hello_zig/hello.wasm")?
        }
        "c" => {
            println!("=== compiling the C plugin");
            #[cfg(feature = "abi_unknown")]
            println!("cfg(abi_unknown) has now effect for C example");
            #[cfg(feature = "abi_wasi")]
            println!("cfg(abi_wasi) has now effect for C example");

            println!("{}", std::env::current_dir().unwrap().display());
            Command::new("emcc")
                .arg("--no-entry")
                .arg("-s")
                .arg("ERROR_ON_UNDEFINED_SYMBOLS=0")
                .arg("-o")
                .arg("hello.wasm")
                .arg("hello.c")
                .current_dir("../hello_c/")
                .spawn()
                .expect("do you have emcc installed and in the path?")
                .wait()?;
            println!("===");
            std::fs::read("../hello_c/hello.wasm")?
        }

        #[cfg(not(any(feature = "abi_unknown", feature = "abi_wasi")))]
        "rust" | "zig" => {
            panic!(
                "for testing rust or zig, you must enable one feature in [abi_unknown, abi_wasi]"
            )
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
