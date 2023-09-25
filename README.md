# wasm-minimal-protocol

A minimal protocol to write [typst plugins](https://typst.app/docs/reference/foundations/plugin/).

Note that plugins require typst version `0.8` or more.

## You want to write a plugin

A plugin can be written in Rust, C, Zig, or any language than compiles to WebAssembly.

Rust plugins can use this crate to automatically implement the protocol with a macro:

```rust
// Rust file
use wasm_minimal_protocol::*;

initiate_protocol!();

#[wasm_func]
pub fn hello() -> Vec<u8> {
    b"Hello from wasm!!!".to_vec()
}
```

```typst
// Typst file
#let p = plugin("/path/to/plugin.wasm")
#assert.eq(str(p.hello()), "Hello from wasm!!!")
```

For other languages, the protocol is described at <https://typst.app/docs/reference/foundations/plugin/>. You should also take a look at this repository's [examples](#examples).

## Examples

See the example for your language:

- [Rust](examples/hello_rust/)
- [Zig](examples/hello_zig/)
- [C](examples/hello_c/)

If you have all the required dependencies, you may build all examples by running `cargo test`.

## wasi-stub

The runtime used by typst do not allow the plugin to import any function (beside the ones used by the protocol). In particular, if your plugin is compiled for [WASI](https://wasi.dev/), it will not be able to be loaded by typst.

To get around that, you can use [wasi-stub](./wasi-stub). It will detect all WASI-related imports, and replace them by stubs that do nothing.

If you are compiling C code with `emcc`, stubbing is almost certainly required.
