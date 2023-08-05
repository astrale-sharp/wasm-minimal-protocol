// this is an example of host
// you need to build the hello example first

use anyhow::Result;
use host_wasmi::PluginInstance;
use std::{path::Path, process::Command};

const WASI: bool = {
    #[cfg(feature = "wasi")]
    {
        true
    }
    #[cfg(not(feature = "wasi"))]
    {
        false
    }
};

fn main() -> Result<()> {
    let mut custom_run = false;
    let args: Vec<_> = std::env::args().skip(1).collect();
    if args.is_empty() {
        anyhow::bail!("1 argument required: 'rust', 'zig' or 'c'")
    }
    if WASI {
        println!("[INFO] The WASI functions will be stubbed (by `wasi-stub`) for this run");
    }
    let mut plugin_binary = match args[0].as_str() {
        "rust" => {
            println!("=== compiling the Rust plugin");
            let result = compile_rust(WASI, ".")?;
            println!("===");
            println!(
                "[INFO] getting wasm from: {}",
                if WASI {
                    "examples/hello_rust/target/wasm32-wasi/debug/hello.wasm"
                } else {
                    "examples/hello_rust/target/wasm32-unknown-unknown/debug/hello.wasm"
                }
            );
            result
        }
        "zig" => {
            println!("=== compiling the Zig plugin");
            let result = compile_zig(WASI, ".")?;
            println!("===");
            println!("[INFO] getting wasm from: examples/hello_zig/hello.wasm");
            result
        }
        "c" => {
            println!("=== compiling the C plugin");
            if !WASI {
                eprintln!("WARNING: the C example should be compiled with `--features wasi`");
            }

            println!("{}", std::env::current_dir().unwrap().display());
            let result = compile_c(".")?;
            println!("===");
            println!("[INFO] getting wasm from: examples/hello_c/hello.wasm");
            result
        }
        "-i" | "--input" => {
            custom_run = true;
            println!("[INFO] getting wasm from: {}", args[1].as_str());
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

    if WASI {
        plugin_binary = {
            println!("[INFO] Using wasi-stub");
            let res = wasi_stub::stub_wasi_functions(&plugin_binary)?;
            println!("[INFO] WASI functions have been stubbed");
            res
        };
    }

    let mut plugin_instance = PluginInstance::new_from_bytes(plugin_binary).unwrap();
    if custom_run {
        let function = args[2].as_str();
        let args = args
            .iter()
            .skip(3)
            .map(|x| x.as_bytes())
            .collect::<Vec<_>>();
        let result = match plugin_instance.call(function, &args) {
            Ok(res) => res,
            Err(err) => {
                eprintln!("Error: {err}");
                return Ok(());
            }
        };
        match std::str::from_utf8(result.get()) {
            Ok(s) => println!("{s}"),
            Err(_) => panic!("Error: function call '{function}' did not return UTF-8"),
        }
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
                eprintln!("Error: {err}");
                continue;
            }
        };
        match std::str::from_utf8(result.get()) {
            Ok(s) => println!("{s}"),
            Err(_) => panic!("Error: function call '{function}' did not return UTF-8"),
        }
    }

    Ok(())
}

fn compile_rust(wasi: bool, dir_root: &str) -> Result<Vec<u8>> {
    let target = if wasi {
        "wasm32-wasi"
    } else {
        "wasm32-unknown-unknown"
    };
    Command::new("cargo")
        .arg("build")
        .arg("--target")
        .arg(target)
        .current_dir(Path::new(dir_root).join("examples/hello_rust"))
        .spawn()
        .map_err(|err| anyhow::format_err!("while spawning the build command: {err}"))?
        .wait()?;
    let path = Path::new(dir_root).join(format!(
        "examples/hello_rust/target/{target}/debug/hello.wasm"
    ));
    std::fs::read(&path)
        .map_err(|err| anyhow::format_err!("while reading {}: {err}", path.display()))
}

fn compile_zig(wasi: bool, dir_root: &str) -> Result<Vec<u8>> {
    Command::new("zig")
        .arg("build-lib")
        .arg("hello.zig")
        .arg("-target")
        .arg(if wasi {
            "wasm32-wasi"
        } else {
            "wasm32-freestanding"
        })
        .arg("-dynamic")
        .arg("-rdynamic")
        .current_dir(Path::new(dir_root).join("examples/hello_zig"))
        .spawn()
        .expect("do you have zig installed and in the path?")
        .wait()?;
    let path = Path::new(dir_root).join("examples/hello_zig/hello.wasm");
    std::fs::read(&path)
        .map_err(|err| anyhow::format_err!("while reading {}: {err}", path.display()))
}

fn compile_c(dir_root: &str) -> Result<Vec<u8>> {
    Command::new("emcc")
        .arg("--no-entry")
        .arg("-s")
        .arg("ERROR_ON_UNDEFINED_SYMBOLS=0")
        .arg("-o")
        .arg("hello.wasm")
        .arg("hello.c")
        .current_dir(Path::new(dir_root).join("examples/hello_c/"))
        .spawn()
        .expect("do you have emcc installed and in the path?")
        .wait()?;
    let path = Path::new(dir_root).join("examples/hello_c/hello.wasm");
    std::fs::read(&path)
        .map_err(|err| anyhow::format_err!("while reading {}: {err}", path.display()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Mutex;

    /// Needed to avoid tests walking all over each others.
    static LOCK: Mutex<()> = Mutex::new(());
    const ROOT_DIR: &str = "../..";

    fn run_function(
        plugin_instance: &mut PluginInstance,
        function: &str,
        args: &[&[u8]],
    ) -> Result<String, String> {
        match plugin_instance.call(function, args) {
            Ok(res) => match std::str::from_utf8(res.get()) {
                Ok(s) => Ok(s.to_owned()),
                Err(_) => panic!("Error: function call '{function}' did not return UTF-8"),
            },
            Err(err) => Err(err),
        }
    }

    fn test_default_functions(plugin_instance: &mut PluginInstance) -> bool {
        let mut all_passed = true;
        for (function, args, expected) in [
            ("hello", &[] as &[&[u8]], Ok("Hello from wasm!!!")),
            ("double_it", &[b"double me!!"], Ok("double me!!double me!!")),
            ("concatenate", &[b"val1", b"value2"], Ok("val1*value2")),
            (
                "shuffle",
                &[b"value1", b"value2", b"value3"],
                Ok("value3-value1-value2"),
            ),
            ("returns_ok", &[], Ok("This is an `Ok`")),
            (
                "returns_err",
                &[],
                Err("plugin errored with: 'This is an `Err`'"),
            ),
            ("will_panic", &[], Err("plugin panicked")),
        ] {
            let res = run_function(plugin_instance, function, args);
            let res = res.as_ref().map(|s| s.as_str()).map_err(|s| s.as_str());
            if expected != res {
                all_passed = false;
                eprintln!("Incorrect result when calling {}:", function);
                eprintln!("  - expected: {:?}", expected);
                eprintln!("  - got: {:?}", res);
            } else {
                println!("calling {function} returned: {:?}", res)
            }
        }
        all_passed
    }

    #[test]
    fn rust_no_wasi() -> Result<()> {
        let lock = LOCK.lock();
        let binary = compile_rust(false, ROOT_DIR)?;
        drop(lock);
        let mut plugin_instance = PluginInstance::new_from_bytes(binary).unwrap();
        if !test_default_functions(&mut plugin_instance) {
            anyhow::bail!("Some incorrect result detected");
        } else {
            Ok(())
        }
    }

    #[test]
    fn rust_wasi() -> Result<()> {
        let lock = LOCK.lock();
        let binary = compile_rust(true, ROOT_DIR)?;
        drop(lock);
        let binary = wasi_stub::stub_wasi_functions(&binary)?;
        let mut plugin_instance = PluginInstance::new_from_bytes(binary).unwrap();
        if !test_default_functions(&mut plugin_instance) {
            anyhow::bail!("Some incorrect result detected");
        } else {
            Ok(())
        }
    }

    #[test]
    fn zig_no_wasi() -> Result<()> {
        let lock = LOCK.lock();
        let binary = compile_zig(false, ROOT_DIR)?;
        drop(lock);
        let mut plugin_instance = PluginInstance::new_from_bytes(binary).unwrap();
        if !test_default_functions(&mut plugin_instance) {
            anyhow::bail!("Some incorrect result detected");
        } else {
            Ok(())
        }
    }

    #[test]
    fn zig_wasi() -> Result<()> {
        let lock = LOCK.lock();
        let binary = compile_zig(true, ROOT_DIR)?;
        drop(lock);
        let binary = wasi_stub::stub_wasi_functions(&binary)?;
        let mut plugin_instance = PluginInstance::new_from_bytes(binary).unwrap();
        if !test_default_functions(&mut plugin_instance) {
            anyhow::bail!("Some incorrect result detected");
        } else {
            Ok(())
        }
    }

    #[test]
    fn c_wasi() -> Result<()> {
        let lock = LOCK.lock();
        let binary = compile_c(ROOT_DIR)?;
        drop(lock);
        let binary = wasi_stub::stub_wasi_functions(&binary)?;
        let mut plugin_instance = PluginInstance::new_from_bytes(binary).unwrap();
        if !test_default_functions(&mut plugin_instance) {
            anyhow::bail!("Some incorrect result detected");
        } else {
            Ok(())
        }
    }
}
