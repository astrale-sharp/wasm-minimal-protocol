# Rust wasm plugin example

This is a bare-bone typst plugin, written in Rust. It uses the [wasm-minimal-protocol](../../) crate to easily define plugin functions.

## Compile

To compile this example, you need to have a working [Rust toolchain](https://www.rust-lang.org/). Then we need to install the `wasm32-unknown-unknown` target:

```sh
rustup target add wasm32-unknown-unknown
```

Then, build the crate with this target:

```sh
cargo build --release --target wasm32-unknown-unknown
cp ./target/wasm32-unknown-unknown/release/hello.wasm ./
```

## Compile with wasi

If you want to build with WASI, use the `wasm32-wasip1` target:

```sh
rustup target add wasm32-wasip1
cargo build --release --target wasm32-wasip1
cp ./target/wasm32-wasip1/release/hello.wasm ./
```

Then, stub the resulting binary:

```sh
cargo run --manifest-path ../../crates/wasi-stub/Cargo.toml hello.wasm -o hello.wasm
```

## Build with typst

Simply run `typst compile hello.typ`, and observe that it works !
