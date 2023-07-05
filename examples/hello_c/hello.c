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

EMSCRIPTEN_KEEPALIVE
int32_t concatenate(size_t arg1_len, size_t arg2_len) {
  size_t total_len = arg1_len + arg2_len;
  uint8_t *args = (uint8_t *)malloc(total_len);
  uint8_t *result = (uint8_t *)malloc(total_len + 1);
  if (args == NULL) {
    return 1;
  } else if (result == NULL) {
    free(args);
    return 1;
  }

  wasm_minimal_protocol_write_args_to_buffer(args);
  uint8_t *arg1 = args;
  uint8_t *arg2 = args + arg1_len;

  for (size_t i = 0; i < arg1_len; i++) {
    result[i] = arg1[i];
  }
  result[arg1_len] = '*';
  for (size_t i = 0; i < arg2_len; i++) {
    result[arg1_len + 1 + i] = arg2[i];
  }

  wasm_minimal_protocol_send_result_to_host(result, total_len + 1);

  free(result);
  free(args);
  return 0;
}

EMSCRIPTEN_KEEPALIVE
int32_t shuffle(size_t arg1_len, size_t arg2_len, size_t arg3_len) {
  size_t result_len = arg1_len + arg2_len + arg3_len + 2;
  uint8_t *args = (uint8_t *)malloc(arg1_len + arg2_len + arg3_len);
  uint8_t *result = (uint8_t *)malloc(result_len);
  if (args == NULL) {
    return 1;
  } else if (result == NULL) {
    free(args);
    return 1;
  }

  wasm_minimal_protocol_write_args_to_buffer(args);
  uint8_t *arg1 = args;
  uint8_t *arg2 = args + arg1_len;
  uint8_t *arg3 = args + arg1_len + arg2_len;

  for (size_t i = 0; i < arg3_len; i++) {
    result[i] = arg3[i];
  }
  result[arg3_len] = '-';
  for (size_t i = 0; i < arg1_len; i++) {
    result[arg3_len + 1 + i] = arg1[i];
  }
  result[arg3_len + arg1_len + 1] = '-';
  for (size_t i = 0; i < arg2_len; i++) {
    result[arg3_len + arg1_len + 2 + i] = arg2[i];
  }

  wasm_minimal_protocol_send_result_to_host(result, result_len);

  free(result);
  free(args);
  return 0;
}

EMSCRIPTEN_KEEPALIVE
int32_t returns_ok() {
  const char message[] = "This is an `Ok`";
  wasm_minimal_protocol_send_result_to_host((uint8_t *)message,
                                            sizeof(message) - 1);
  return 0;
}

EMSCRIPTEN_KEEPALIVE
int32_t returns_err() {
  const char message[] = "This is an `Err`";
  wasm_minimal_protocol_send_result_to_host((uint8_t *)message,
                                            sizeof(message) - 1);
  return 1;
}

// Needs WASI
// EMSCRIPTEN_KEEPALIVE
// int32_t will_panic() { exit(1); }