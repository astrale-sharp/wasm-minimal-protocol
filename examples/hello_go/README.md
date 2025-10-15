# Go wasm plugin example

This is a bare-bone typst plugin, written in Go.

## Compile

To compile this example, you need the [TinyGo compiler](https://tinygo.org/). Then, run the command:

```sh
tinygo build -target=wasm-unknown -o hello.wasm
```

The `-target=wasm-unknown` flag is similar to `-target wasm32-unknown-unknown` in Rust and it tells the compiler to generate a WebAssembly binary that does not depend on any specific environment.

## Compile with wasip1

If you want to build with WASI, use the `wasip1` target:

```sh
GOOS=wasip1 GOARCH=wasm tinygo build -o hello.wasm
```

Then, stub the resulting binary:

```sh
cargo run --manifest-path ../../crates/wasi-stub/Cargo.toml hello.wasm -o hello.wasm
```

## Build with typst

Simply run `typst compile hello.typ`, and observe that it works!
