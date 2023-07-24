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
    pub const RUST_PATH: &str =
        "examples/hello_rust/target/wasm32-unknown-unknown/debug/hello.wasm";
    pub const ZIG_TARGET: &str = "wasm32-freestanding";
}

#[cfg(feature = "abi_wasi")]
mod consts {
    pub const RUST_TARGET: &str = "wasm32-wasi";
    pub const RUST_PATH: &str = "examples/hello_rust/target/wasm32-wasi/debug/hello.wasm";
    pub const ZIG_TARGET: &str = "wasm32-wasi";
}

#[cfg(any(
    all(feature = "host-wasmtime", feature = "host-wasmer"),
    all(feature = "host-wasmtime", feature = "host-wasmi"),
    all(feature = "host-wasmer", feature = "host-wasmi"),
))]
compile_error!("Only one feature in [host-wasmtime, host-wasmi, host-wasmer] must be specified. host-wasmi is enabled by default.");

#[cfg(not(any(
    feature = "host-wasmtime",
    feature = "host-wasmer",
    feature = "host-wasmi"
)))]
compile_error!(
    "At least one feature in [host-wasmtime, host-wasmi, host-wasmer] must be specified."
);

fn main() -> Result<()> {
    let mut custom_run = false;
    let args: Vec<_> = std::env::args().skip(1).collect();
    if args.is_empty() {
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
                .current_dir("examples/hello_rust")
                .spawn()?
                .wait()?;
            println!("===");
            println!("getting wasm from: {}", consts::RUST_PATH);
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
                .current_dir("examples/hello_zig")
                .spawn()
                .expect("do you have zig installed and in the path?")
                .wait()?;
            println!("===");
            println!("getting wasm from: examples/hello_zig/hello.wasm");
            std::fs::read("examples/hello_zig/hello.wasm")?
        }
        "c" => {
            println!("=== compiling the C plugin");
            #[cfg(feature = "abi_unknown")]
            println!("cfg(abi_unknown) has no effect for C example");
            #[cfg(feature = "abi_wasi")]
            println!("cfg(abi_wasi) has no effect for C example");

            println!("{}", std::env::current_dir().unwrap().display());
            Command::new("emcc")
                .arg("--no-entry")
                .arg("-s")
                .arg("ERROR_ON_UNDEFINED_SYMBOLS=0")
                .arg("-o")
                .arg("hello.wasm")
                .arg("hello.c")
                .current_dir("examples/hello_c/")
                .spawn()
                .expect("do you have emcc installed and in the path?")
                .wait()?;
            println!("===");
            println!("getting wasm from: examples/hello_c/hello.wasm");
            std::fs::read("examples/hello_c/hello.wasm")?
        }

        #[cfg(not(any(feature = "abi_unknown", feature = "abi_wasi")))]
        "rust" | "zig" => {
            panic!(
                "for testing rust or zig, you must enable one feature in [abi_unknown, abi_wasi]"
            )
        }
        "-i" | "--input" => {
            custom_run = true;
            println!("===");
            println!("getting wasm from: {}", args[1].as_str());
            println!(
                "running func: {}",
                args.get(2)
                    .expect("you must specify a function to run")
                    .as_str()
            );
            std::fs::read(args[1].as_str())?
        }
        _ => anyhow::bail!("unknown argument '{}'", args[0].as_str()),
    };

    let mut plugin_instance = PluginInstance::new_from_bytes(plugin_binary).unwrap();
    if custom_run {
        println!(
            "{:?}",
            plugin_instance.call(
                args[2].as_str(),
                args.iter()
                    .skip(3)
                    .map(|x| x.as_bytes())
                    .collect::<Vec<_>>()
                    .as_slice()
            )
        );
        return Ok(());
    }

    for (function, args) in [
        ("hello", &[] as &[&[u8]]),
        ("double_it", &[b"double me!!"]),
        ("concatenate", &[b"val1", b"value2"]),
        ("shuffle", &[b"value1", b"value2", b"value3"]),
        ("returns_ok", &[]),
        ("returns_err", &[]),
        ("will_panic", &[]),
    ] {
        let result = match plugin_instance.call(function, args) {
            Ok(res) => res,
            Err(err) => {
                println!("Error: {err}");
                continue;
            }
        };
        match String::from_utf8(result) {
            Ok(s) => println!("{s}"),
            Err(_) => panic!("Error: function call '{function}' did not return UTF-8"),
        }
    }

    Ok(())
}
