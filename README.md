# wasm-minimal-protocol
A minimal protocol to send/receive strings from wasm while doing very little/none fancy things. 
Primarily developed to interface with the [typst language.](https://typst.app/).

Your wasm returns a code (0 if everything went fine)

## You want to write a plugin
See the rust example here for  [Rust](examples/hello_rust/) [Zig](examples/hello_zig/) or [C](examples/hello_c/)

## You want to run tests, seing what these plugins do
- hosts have been implemented using wasmi, wasmer and wasmtime, you must specify a feature to test one of them
- you must also specify the abi (unknown or wasi)
Examples:
- `cargo run -p test-runner --features host-wasmi,abi_unknown -- zig`
- `cargo run -p test-runner --features host-wasmer,abi_wasi -- rust`
- `cargo run -p test-runner --features host-wasmtime -- c`


Your plugin may be compiled to the target wasm32-wasi if it's easier for you but true support for wasmi may not be present or easy to support, this question is pending. (host-wasmi doesn't support it yet)


## Tips
- If you run into error about snapshot-preview etc, you should try using [this project](https://github.com/near/wasi-stub) on your wasm file. It stubs all wasi function in your wasm, don't expect print or read_file to work anymore.