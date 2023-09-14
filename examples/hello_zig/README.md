# Zig wasm plugin example

This is a bare-bone typst plugin, written in Zig.

## Compile

To compile this example, you need the [zig compiler](https://ziglang.org/learn/getting-started/#installing-zig). Then, run the command

```sh
zig build-lib hello.zig -target wasm32-freestanding -dynamic -rdynamic -O ReleaseSmall
```

## Compile with wasi

If you want to build with WASI, use the `wasm32-wasi` target:

```sh
zig build-lib hello.zig -target wasm32-wasi -dynamic -rdynamic -O ReleaseSmall -femit-bin=hello-wasi.wasm
```

Then, stub the resulting binary:

```sh
pushd ../../wasi-stub
cargo run -- ../examples/hello_zig/hello-wasi.wasm -o ../examples/hello_zig/hello-wasi.wasm
popd
```

## Build with typst

Simply run `typst compile hello.typ`, and observe that it works !