#include "emscripten.h"

#ifdef __cplusplus
#include <cstddef>
#include <cstdint>
#include <cstdlib>
#include <cstring>
#define PROTOCOL_FUNCTION __attribute__((import_module("typst_env"))) extern "C"
#else
#include <stddef.h>
#include <stdint.h>
#include <stdlib.h>
#include <string.h>
#define PROTOCOL_FUNCTION __attribute__((import_module("typst_env"))) extern
#endif

// ===
// Functions for the protocol

PROTOCOL_FUNCTION void
wasm_minimal_protocol_send_result_to_host(const uint8_t *ptr, size_t len);
PROTOCOL_FUNCTION void wasm_minimal_protocol_write_args_to_buffer(uint8_t *ptr);

// ===

EMSCRIPTEN_KEEPALIVE
int32_t hello(void) {
  const char message[] = "Hello from wasm!!!";
  const size_t length = sizeof(message);
  wasm_minimal_protocol_send_result_to_host((const uint8_t *)message, length - 1);
  return 0;
}

EMSCRIPTEN_KEEPALIVE
int32_t double_it(size_t arg_len) {
  size_t result_len = arg_len * 2;
  uint8_t *result = (uint8_t *)malloc(result_len);
  if (result == NULL) {
    return 1;
  }
  wasm_minimal_protocol_write_args_to_buffer(result);
  memcpy(result + arg_len, result, arg_len);
  wasm_minimal_protocol_send_result_to_host(result, result_len);
  free(result);
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

  memcpy(result, arg1, arg1_len);
  result[arg1_len] = '*';
  memcpy(result + arg1_len + 1, arg2, arg2_len);

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

  memcpy(result, arg3, arg3_len);
  result[arg3_len] = '-';
  memcpy(result + arg3_len + 1, arg1, arg1_len);
  result[arg3_len + 1 + arg1_len] = '-';
  memcpy(result + arg3_len + 1 + arg1_len + 1, arg2, arg2_len);

  wasm_minimal_protocol_send_result_to_host(result, result_len);

  free(result);
  free(args);
  return 0;
}

EMSCRIPTEN_KEEPALIVE
int32_t returns_ok() {
  const char message[] = "This is an `Ok`";
  const size_t length = sizeof(message);
  wasm_minimal_protocol_send_result_to_host((const uint8_t *)message, length - 1);
  return 0;
}

EMSCRIPTEN_KEEPALIVE
int32_t returns_err() {
  const char message[] = "This is an `Err`";
  const size_t length = sizeof(message);
  wasm_minimal_protocol_send_result_to_host((const uint8_t *)message, length - 1);
  return 1;
}

EMSCRIPTEN_KEEPALIVE
int32_t will_panic() {
  exit(1);
}
