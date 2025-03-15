const std = @import("std");
const allocator = std.heap.page_allocator;

// ===
// Functions for the protocol

extern "typst_env" fn wasm_minimal_protocol_send_result_to_host(ptr: [*]const u8, len: usize) void;
extern "typst_env" fn wasm_minimal_protocol_write_args_to_buffer(ptr: [*]u8) void;

// ===

export fn hello() i32 {
    const message = "Hello from wasm!!!";
    wasm_minimal_protocol_send_result_to_host(message.ptr, message.len);
    return 0;
}

export fn double_it(arg1_len: usize) i32 {
    var result = allocator.alloc(u8, arg1_len * 2) catch return 1;
    defer allocator.free(result);
    wasm_minimal_protocol_write_args_to_buffer(result.ptr);
    for (0..arg1_len) |i| {
        result[i + arg1_len] = result[i];
    }
    wasm_minimal_protocol_send_result_to_host(result.ptr, result.len);
    return 0;
}

export fn concatenate(arg1_len: usize, arg2_len: usize) i32 {
    const args = allocator.alloc(u8, arg1_len + arg2_len) catch return 1;
    defer allocator.free(args);
    wasm_minimal_protocol_write_args_to_buffer(args.ptr);

    var result = allocator.alloc(u8, arg1_len + arg2_len + 1) catch return 1;
    defer allocator.free(result);
    for (0..arg1_len) |i| {
        result[i] = args[i];
    }
    result[arg1_len] = '*';
    for (arg1_len..arg1_len + arg2_len) |i| {
        result[i + 1] = args[i];
    }
    wasm_minimal_protocol_send_result_to_host(result.ptr, result.len);
    return 0;
}

export fn shuffle(arg1_len: usize, arg2_len: usize, arg3_len: usize) i32 {
    const args_len = arg1_len + arg2_len + arg3_len;
    var args = allocator.alloc(u8, args_len) catch return 1;
    defer allocator.free(args);
    wasm_minimal_protocol_write_args_to_buffer(args.ptr);

    const arg1 = args[0..arg1_len];
    const arg2 = args[arg1_len .. arg1_len + arg2_len];
    const arg3 = args[arg1_len + arg2_len .. args.len];

    var result = allocator.alloc(u8, arg1_len + arg2_len + arg3_len + 2) catch return 1;
    defer allocator.free(result);
    @memcpy(result[0..arg3.len], arg3);
    result[arg3.len] = '-';
    @memcpy(result[arg3.len + 1 ..][0..arg1.len], arg1);
    result[arg3.len + arg1.len + 1] = '-';
    @memcpy(result[arg3.len + arg1.len + 2 ..][0..arg2.len], arg2);

    wasm_minimal_protocol_send_result_to_host(result.ptr, result.len);
    return 0;
}

export fn returns_ok() i32 {
    const message = "This is an `Ok`";
    wasm_minimal_protocol_send_result_to_host(message.ptr, message.len);
    return 0;
}

export fn returns_err() i32 {
    const message = "This is an `Err`";
    wasm_minimal_protocol_send_result_to_host(message.ptr, message.len);
    return 1;
}

export fn will_panic() i32 {
    std.debug.panic("Panicking, this message will not be seen...", .{});
}
