# C wasm plugin example

This is a bare-bone typst plugin, written in C.

## Compile

To compile this example, you need the [emcc compiler](https://emscripten.org/docs/getting_started/downloads.html). Then, run the command

```sh
emcc --no-entry -O3 -s ERROR_ON_UNDEFINED_SYMBOLS=0 -o hello.wasm hello.c
```

Emcc always build with WASI, so we need to stub WASI functions:

```sh
pushd ../../wasi-stub
cargo run -- ../examples/hello_c/hello.wasm -o ../examples/hello_c/hello.wasm
popd
```

## Build with typst

Simply run `typst compile hello.typ`, and observe that it works !