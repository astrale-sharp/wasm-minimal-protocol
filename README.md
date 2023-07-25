# wasm-minimal-protocol

A minimal protocol to send/receive messages from wasm.
Primarily developed to interface with the [typst language](https://typst.app/).

- [wasm-minimal-protocol](#wasm-minimal-protocol)
  - [You want to write a plugin](#you-want-to-write-a-plugin)
  - [Examples](#examples)
    - [Dependencies](#dependencies)
    - [Some commands](#some-commands)
    - [Tips](#tips)
  - [You need to stub a WebAssembly plugin](#you-need-to-stub-a-webassembly-plugin)

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

The example can run using [`wasmi`](https://github.com/paritytech/wasmi).

The command to run examples (from the top-level directory) is:

```sh
cargo run [--features wasi] -- <lang>
# or
cargo run [--features wasi] -- -i <PATH> <func> <args>
```

Where:

- `<lang>` is `rust`, `zig` or `c`
- add `wasi` to the list of features to compile the example with WASI (required on the C example) and stub all the resulting WASI function if the runner is `host-wasmi`.
- `<PATH>` is the path to a wasm file
- `<func>` is the exported function to call in the wasm file, with `<args>` as arguments

### Dependencies

- All commands require a valid [Rust toolchain](https://www.rust-lang.org/).
- The Zig example requires the [Zig toolchain](https://ziglang.org/learn/getting-started/#installing-zig).
- The C example requires [emscripten](https://emscripten.org/docs/getting_started/downloads.html).

### Some commands

```sh
cargo run -- rust # compile and run the Rust example
cargo run --features wasi -- rust # compile and run the Rust example with WASI (stubbed)
cargo run -- zig # compile and run the Zig example
# NOTE: this needs the wasi feature, as `emcc` only compiles in WASI.
cargo run --features wasi -- c # compile and run the C example
cargo run -- -i MY_WASM_FILE.wasm MY_FUNCTION arg1 arg2
```

### Tips

- `host-wasmi` does not support running with WASI, and will stub all WASI functions instead.
- host-wasmi compiles fastest 😉

## You need to stub a WebAssembly plugin

We got you covered, take a look [here](wasi-stub/README.md)
