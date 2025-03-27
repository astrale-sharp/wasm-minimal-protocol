use std::{
    path::{Path, PathBuf},
    process::Command,
};

fn wasi_stub(path: PathBuf) {
    let path = path.canonicalize().unwrap();

    let wasi_stub = Command::new("cargo")
        .arg("run")
        .arg(&path)
        .arg("-o")
        .arg(&path)
        .current_dir(concat!(env!("CARGO_MANIFEST_DIR"), "/../wasi-stub"))
        .status()
        .unwrap();
    if !wasi_stub.success() {
        panic!("wasi-stub failed");
    }
}

fn typst_compile(path: &Path) {
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
    let dir_path = Path::new(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../examples/hello_c"
    ));

    let build_c = Command::new("emcc")
        .arg("--no-entry")
        .arg("-O3")
        .arg("-s")
        .arg("ERROR_ON_UNDEFINED_SYMBOLS=0")
        .arg("-o")
        .arg("hello.wasm")
        .arg("hello.c")
        .current_dir(dir_path)
        .status()
        .unwrap();
    if !build_c.success() {
        panic!("Compiling with emcc failed");
    }
    wasi_stub(dir_path.join("hello.wasm"));
    typst_compile(dir_path);
}

#[test]
fn test_rust() {
    let dir_path = Path::new(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../examples/hello_rust"
    ));

    for target in ["wasm32-unknown-unknown", "wasm32-wasip1"] {
        let build_rust = Command::new("cargo")
            .arg("build")
            .arg("--release")
            .arg("--target")
            .arg(target)
            .current_dir(dir_path)
            .status()
            .unwrap();
        if !build_rust.success() {
            panic!("Compiling with cargo failed");
        }
        std::fs::copy(
            dir_path
                .join("target")
                .join(target)
                .join("release/hello.wasm"),
            dir_path.join("hello.wasm"),
        )
        .unwrap();
        if target == "wasm32-wasip1" {
            wasi_stub(dir_path.join("hello.wasm"));
        }
        typst_compile(dir_path);
    }
}

#[test]
fn test_zig() {
    let dir_path = Path::new(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../examples/hello_zig"
    ));

    for target in ["wasm32-freestanding", "wasm32-wasi"] {
        let build_zig = Command::new("zig")
            .arg("build-exe")
            .arg("hello.zig")
            .arg("-target")
            .arg(target)
            .arg("-fno-entry")
            .arg("-O")
            .arg("ReleaseSmall")
            .arg("--export=hello")
            .arg("--export=double_it")
            .arg("--export=concatenate")
            .arg("--export=shuffle")
            .arg("--export=returns_ok")
            .arg("--export=returns_err")
            .arg("--export=will_panic")
            .current_dir(dir_path)
            .status()
            .unwrap();
        if !build_zig.success() {
            panic!("Compiling with zig failed");
        }
        if target == "wasm32-wasi" {
            wasi_stub(dir_path.join("hello.wasm"));
        }
        typst_compile(dir_path);
    }
}
