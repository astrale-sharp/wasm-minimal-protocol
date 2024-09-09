# Go wasm plugin example

This is a bare-bone typst plugin, written in Go.

## Compile

To compile this example, you need the [TinyGo compiler]. Then, run the command

```sh
GOOS="wasip1" GOARCH="wasm" tinygo build -o hello.wasm
```

[TinyGo compiler]: https://tinygo.org/getting-started/install/


This invocation of Tinygo builds with WASI[^1], so we need to stub WASI functions:

[^1]: I personally could not get a working binary with `GOOS="js"` (i.e. wasm).

```sh
pushd ../../wasi-stub
cargo run -- ../examples/hello_go/hello.wasm -o ../examples/hello_go/hello.wasm
popd
```

## Build with typst

Simply run `typst compile hello.typ`, and observe that it works !
