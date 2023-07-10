# wasm-minimal-protocol
A minimal protocol to send/receive strings from wasm while doing very little/none fancy things. 
Primarily developed to interface with the [typst language.](https://typst.app/).

Your wasm returns a code (0 if everything went fine)

## You want to write a plugin
See the rust example here for  [Rust](examples/hello_rust/), [Zig](examples/hello_zig/) or [C](examples/hello_c/)

## You want to run tests, seing what these plugins do
- hosts have been implemented using wasmi, wasmer and wasmtime, the default is wasmi, you must specify any other one with a feature.
- you must also specify the abi (unknown or wasi), the default is unknown.
- host-wasmi cannot run abi_wasi if the functions have not been stubbed

Examples (run from root of project):
- `cargo run -p test-runner -- zig`
- `cargo run -p test-runner -- rust`
- `cargo run -p test-runner -- c` will fail cause the c hasn't been stub.
- `cargo run -p test-runner --no-default-features --features host-wasmer -- c` will succeed
- `cargo run -p test-runner --no-default-features --features host-wasmtime,abi_wasi -- rust`
- `cargo run -p test-runner --no-default-features --features host-wasmtime,abi_wasi -- zig`
you may also specify --input or -i to choose a file:
`cargo run -p test-runner --features host-wasmi -- -i my_wasm_file.wasm`


Your plugin may be compiled to the target wasm32-wasi if it's easier for you but true support for wasmi may not be present or easy to support, this question is pending. (host-wasmi doesn't support it yet)


## Tips
- If you run into error about snapshot-preview etc, you should try using [wasi-stub](./wasi-stub/) on your wasm file. It stubs all wasi function in your wasm, don't expect print or read_file to work anymore.
- host-wasmi compiles fastest ;)
