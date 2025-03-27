# Zig wasm plugin example

This is a bare-bone typst plugin, written in Zig.

## Compile

To compile this example, you need the [zig compiler](https://ziglang.org/learn/getting-started/#installing-zig). Then, run the command

```sh
zig build-exe hello.zig -target wasm32-freestanding -fno-entry -O ReleaseSmall \
    --export=hello \
    --export=double_it \
    --export=concatenate \
    --export=shuffle \
    --export=returns_ok \
    --export=returns_err \
    --export=will_panic
```

## Compile with wasi

If you want to build with WASI, use the `wasm32-wasi` target:

```sh
zig build-exe hello.zig -target wasm32-wasi -fno-entry -O ReleaseSmall \
    --export=hello \
    --export=double_it \
    --export=concatenate \
    --export=shuffle \
    --export=returns_ok \
    --export=returns_err \
    --export=will_panic
```

Then, stub the resulting binary:

```sh
cargo run --manifest-path ../../crates/wasi-stub/Cargo.toml hello.wasm -o hello.wasm
```

## Build with typst

Simply run `typst compile hello.typ`, and observe that it works !
