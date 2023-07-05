cd examples/hello-zig/
zig build-lib hello.zig -target wasm32-freestanding -dynamic -rdynamic
rm hello.wasm.o
cd ../..
cp examples/hello-zig/hello.wasm ./target/wasm32-unknown-unknown/debug/hello.wasm
cargo run -p host-wasmer