# Tests

To run the tests, you will need:

- The emcc compiler: <https://emscripten.org/docs/getting_started/downloads.html>
- An up-to-date Rust toolchain: <https://www.rust-lang.org/>
- A zig compiler, version `0.11`: <https://ziglang.org/learn/getting-started/#installing-zig>

Then, you can run the tests with `cargo test`.

# Git hooks

To run tests automatically, and check that you don't accidentally bump dependencies of `wasm-minimal-protocol`, add the pre-push [git hook](https://git-scm.com/docs/githooks):
```sh
git config --local core.hooksPath .githooks
```

The script `.githooks/pre-push` will be run each time you `git push` (except if you use `--no-verify`).