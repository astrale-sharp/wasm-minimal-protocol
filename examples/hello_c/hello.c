#include "emscripten.h"

#ifdef __cplusplus
#include <cstddef>
#include <cstdint>
#include <cstdlib>
#define PROTOCOL_FUNCTION __attribute__((import_module("typst_env"))) extern "C"
#else
#include <stddef.h>
#include <stdint.h>
#include <stdlib.h>
#define PROTOCOL_FUNCTION __attribute__((import_module("typst_env"))) extern
#endif

PROTOCOL_FUNCTION void
wasm_minimal_protocol_send_result_to_host(const uint8_t *ptr, size_t len);
PROTOCOL_FUNCTION void wasm_minimal_protocol_write_args_to_buffer(uint8_t *ptr);

EMSCRIPTEN_KEEPALIVE
int32_t hello(void) {
  const char message[] = "Hello world !";
  wasm_minimal_protocol_send_result_to_host((uint8_t *)message,
                                            sizeof(message) - 1);
  return 0;
}

EMSCRIPTEN_KEEPALIVE
int32_t double_it(size_t arg_len) {
  size_t result_len = arg_len * 2;
  uint8_t *alloc_result = (uint8_t *)malloc(result_len);
  if (alloc_result == NULL) {
    return 1;
  }
  wasm_minimal_protocol_write_args_to_buffer(alloc_result);
  for (size_t i = 0; i < arg_len; i++) {
    alloc_result[arg_len + i] = alloc_result[i];
  }
  wasm_minimal_protocol_send_result_to_host(alloc_result, result_len);
  free(alloc_result);
  return 0;
}

// TODO: 'concatenate' function
// TODO: 'shuffle' function
// TODO: 'returns_ok' function
// TODO: 'returns_err' function
// TODO: 'will_panic' function