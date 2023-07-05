# wasm-minimal-protocol
A very minimal protocol to send/receive strings from wasm while doing very little/none fancy things.

you can see an example of how it's used  in [hello](./examples/hello/) and how an host might use it in [host-wasmer](./examples/host-wasmer/)

It's primary goal is exploring the design space for making wasm plugins for the [typst language.](https://typst.app/)

To build and test hello and host wasmer you can run 
- `cargo run -p host-wasmer -- rust` for the Rust version
- `cargo run -p host-wasmer -- zig` for the Zig version