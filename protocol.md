This describes the protocol implemented in this crate. This protocol sends and receive byte slices with an host.

Types and functions are described using WAT syntax.

# Compilation

This protocol is only meant to be used by plugins compiled to 32-bits WebAssembly.

A plugin should compile to a shared WebAssembly library.

# Imports

Valid plugins need to import two functions (that will be provided by the runtime):

- `(import "typst_env" "wasm_minimal_protocol_write_args_to_buffer" (func (param i32)))`

  The argument is a pointer to a buffer (`ptr`).

  Write the arguments for the current function into the buffer pointed at by `ptr`.

  Each function for the protocol receives lengths as its arguments (see [User-defined functions](#user-defined-functions)). The capacity of the buffer pointed at by `ptr` should be at least the sum of all those lengths.

- `(import "typst_env" "wasm_minimal_protocol_send_result_to_host" (func (param i32 i32)))`

  The first parameter is a pointer to a buffer (`ptr`), the second is the length of the buffer (`len`).

  Send `len` and `ptr` to host memory. The buffer must not be freed by the end of the function: it will be freed by the runtime by calling [`wasm_minimal_protocol_send_result_to_host`](#exports).

  If the message should be interpreted as an error message (see [User-defined functions](#user-defined-functions)), it should be encoded as UTF-8.

  ### Note

  If [`wasm_minimal_protocol_send_result_to_host`](#exports) calls `free` (or a similar routine), be careful that the buffer does not point to static memory.

# Exports

Valid plugins need to export a function named `wasm_minimal_protocol_send_result_to_host`, that has signature `func (param i32 i32)`.

This function will be used by the runtime to free the block of memory returned by a [user-defined](#user-defined-functions) function.

# User-defined functions

To conform to the protocol, an exported function should:

- Take `n` arguments `a₁`, `a₂`, ..., `aₙ` of type `u32` (interpreted as lengths, so `usize/size_t` may be preferable), and return one `i32`. We will call the return `return_code`.
- The function should first allocate a buffer `buf` of length `a₁ + a₂ + ⋯ + aₙ`, and call `wasm_minimal_protocol_write_args_to_buffer(buf.ptr)`.
- The `a₁` first bytes of the buffer constitute the first argument, the `a₂` next bytes the second argument, and so on.
- Before returning, the function should call `wasm_minimal_protocol_send_result_to_host` to send its result back to the host.
- To signal success, `return_code` must be `0`.
- To signal an error, `return_code` must be `1`. The sent buffer is then interpreted as an error message, and must be encoded as UTF-8.
