use std::{path::PathBuf, process::Command};

fn wasi_stub(path: String) {
    let path = PathBuf::from(path).canonicalize().unwrap();

    let wasi_stub = Command::new("cargo")
        .arg("run")
        .arg(&path)
        .arg("-o")
        .arg(&path)
        .current_dir("wasi-stub")
        .status()
        .unwrap();
    if !wasi_stub.success() {
        panic!("wasi-stub failed");
    }
}

fn typst_compile(path: &str) {
    let typst_version = Command::new("typst").arg("--version").output().unwrap();
    if !typst_version.status.success() {
        panic!("typst --version failed");
    }
    let version_string = match String::from_utf8(typst_version.stdout) {
        Ok(s) => s,
        Err(err) => panic!("failed to parse typst version: {err}"),
    };
    if let Some(s) = version_string.strip_prefix("typst ") {
        let version = s.split('.').collect::<Vec<_>>();
        let [major, minor, _] = version.as_slice() else {
            panic!("failed to parse version string {version_string}")
        };
        if !(major.parse::<u64>().unwrap() >= 1 || minor.parse::<u64>().unwrap() >= 8) {
            panic!("The typst version is too low for plugin: you need at least 0.8.0");
        }
    }

    let path = PathBuf::from(path).canonicalize().unwrap();
    let typst_compile = Command::new("typst")
        .arg("compile")
        .arg("hello.typ")
        .current_dir(path)
        .status()
        .unwrap();
    if !typst_compile.success() {
        panic!("typst compile failed");
    }
}

#[test]
fn test_c() {
    let dir_path = "examples/hello_c/".to_string();
    let build_c = Command::new("emcc")
        .arg("--no-entry")
        .arg("-O3")
        .arg("-s")
        .arg("ERROR_ON_UNDEFINED_SYMBOLS=0")
        .arg("-o")
        .arg("hello.wasm")
        .arg("hello.c")
        .current_dir(&dir_path)
        .status()
        .unwrap();
    if !build_c.success() {
        panic!("Compiling with emcc failed");
    }
    wasi_stub(dir_path.clone() + "hello.wasm");
    typst_compile(&dir_path);
}

#[test]
fn test_rust() {
    let dir_path = "examples/hello_rust/".to_string();
    let build_rust = Command::new("cargo")
        .arg("build")
        .arg("--release")
        .arg("--target")
        .arg("wasm32-unknown-unknown")
        .current_dir(&dir_path)
        .status()
        .unwrap();
    if !build_rust.success() {
        panic!("Compiling with cargo failed");
    }
    let build_rust_wasi = Command::new("cargo")
        .arg("build")
        .arg("--release")
        .arg("--target")
        .arg("wasm32-wasi")
        .current_dir(&dir_path)
        .status()
        .unwrap();
    if !build_rust_wasi.success() {
        panic!("Compiling with cargo failed");
    }
    std::fs::copy(
        "examples/hello_rust/target/wasm32-unknown-unknown/release/hello.wasm",
        "examples/hello_rust/hello.wasm",
    )
    .unwrap();
    std::fs::copy(
        "examples/hello_rust/target/wasm32-wasi/release/hello.wasm",
        "examples/hello_rust/hello-wasi.wasm",
    )
    .unwrap();
    wasi_stub(dir_path.clone() + "hello-wasi.wasm");
    typst_compile(&dir_path);
}

#[test]
fn test_zig() {
    let dir_path = "examples/hello_zig/".to_string();
    let build_zig = Command::new("zig")
        .arg("build-lib")
        .arg("hello.zig")
        .arg("-target")
        .arg("wasm32-freestanding")
        .arg("-dynamic")
        .arg("-rdynamic")
        .arg("-O")
        .arg("ReleaseSmall")
        .current_dir(&dir_path)
        .status()
        .unwrap();
    if !build_zig.success() {
        panic!("Compiling with zig failed");
    }
    let build_zig_wasi = Command::new("zig")
        .arg("build-lib")
        .arg("hello.zig")
        .arg("-target")
        .arg("wasm32-wasi")
        .arg("-dynamic")
        .arg("-rdynamic")
        .arg("-O")
        .arg("ReleaseSmall")
        .arg("-femit-bin=hello-wasi.wasm")
        .current_dir(&dir_path)
        .status()
        .unwrap();
    if !build_zig_wasi.success() {
        panic!("Compiling with zig failed");
    }
    wasi_stub(dir_path.clone() + "hello-wasi.wasm");
    typst_compile(&dir_path);
}
