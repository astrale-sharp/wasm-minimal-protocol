# wasm-minimal-protocol

A minimal protocol to send/receive messages from wasm.
Primarily developed to interface with the [typst language](https://typst.app/).

## You want to write a plugin

A plugin can be written in Rust, C, Zig, or any language than compiles to WebAssembly.

Rust plugins can use this crate to automatically implement the protocol with a macro:

```rust
use wasm_minimal_protocol::*;

initiate_protocol!();

#[wasm_func]
pub fn hello() -> Vec<u8> {
    b"Hello from wasm!!!".to_vec()
}
```

For others, the protocols is described in the file [protocol.md](./protocol.md). You should also take a look at the [examples](#examples).

## Examples

Examples are implemented in [Rust](examples/hello_rust/), [Zig](examples/hello_zig/) and [C](examples/hello_c/). Each of them is run using the [test-runner](examples/test-runner/).

The example can run using [`wasmi`](https://github.com/paritytech/wasmi), [`wasmer`](https://github.com/wasmerio/wasmer) or [`wasmtime`](https://github.com/bytecodealliance/wasmtime).

The command to run examples (from the top-level directory) is:

```sh
cargo run -- <lang>
# or
cargo run --no-default-features --features <host>,<abi> -- <lang>
# or
cargo run -- -i <PATH> <func> <args>
# or
cargo run --no-default-features --features <host>,<abi> -- -i <PATH> <func> <args>
```

Where:

- `<lang>` is `rust`, `zig` or `c`
- `<host>` is `host-wasmi`, `host-wasmtime` or `host-wasmer` (defaults to `host-wasmi`)
- `<abi>` is `abi_unknown` or `abi_wasi` (defaults to `abi_unknown`)
- `<PATH>` is the path to a wasm file
- `<func>` is the exported function to call in the wasm file, with `<args>` as arguments

### Dependencies

- All commands require a valid [Rust toolchain](https://www.rust-lang.org/).
- The Zig example requires the [Zig toolchain](https://ziglang.org/learn/getting-started/#installing-zig).
- The C example requires [emscripten](https://emscripten.org/docs/getting_started/downloads.html).

### Some commands

```sh
cargo run -- rust # compile and run the Rust example
cargo run -- zig # compile and run the Zig example
# NOTE: this needs the abi_wasi feature, because the wasi functions are not stubbed. See the 'Tips' section to learn more.
cargo run --no-default-features --features host-wasmtime,abi_wasi -- c # compile and run the C example
cargo run -- -i MY_WASM_FILE.wasm MY_FUNCTION arg1 arg2
```

### Tips

- `host-wasmi` does not support running with WASI (and thus `abi_wasi` will have no effects).
- If the runner complains about missing definition for `wasi_snapshot_preview1` functions, try running your `.wasm` through [wasi-stub](./wasi-stub/). It stubs all wasi function in your wasm, so don't expect print or read_file to work anymore.
- host-wasmi compiles fastest 😉
