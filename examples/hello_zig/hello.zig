const std = @import("std");
const allocator = std.heap.page_allocator;

extern "typst_env" fn wasm_minimal_protocol_send_result_to_host(ptr: [*]const u8, len: usize) void;
extern "typst_env" fn wasm_minimal_protocol_write_args_to_buffer(ptr: [*]u8) void;

export fn hello() i32 {
    const message = "Hello world !";
    wasm_minimal_protocol_send_result_to_host(message.ptr, message.len);
    return 0;
}

export fn double_it(arg1_len: usize) i32 {
    var alloc_result = allocator.alloc(u8, arg1_len * 2) catch return 1;
    defer allocator.free(alloc_result);
    wasm_minimal_protocol_write_args_to_buffer(alloc_result.ptr);
    for (0..arg1_len) |i| {
        alloc_result[i + arg1_len] = alloc_result[i];
    }
    wasm_minimal_protocol_send_result_to_host(alloc_result.ptr, alloc_result.len);
    return 0;
}

export fn concatenate(arg1_len: usize, arg2_len: usize) i32 {
    var args = allocator.alloc(u8, arg1_len + arg2_len) catch return 1;
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
    var args_len = arg1_len + arg2_len + arg3_len;
    var args = allocator.alloc(u8, args_len) catch return 1;
    defer allocator.free(args);
    wasm_minimal_protocol_write_args_to_buffer(args.ptr);

    var arg1 = args[0..arg1_len];
    var arg2 = args[arg1_len .. arg1_len + arg2_len];
    var arg3 = args[arg1_len + arg2_len .. args.len];

    var result: std.ArrayList(u8) = std.ArrayList(u8).initCapacity(allocator, args_len + 2) catch return 1;
    defer result.deinit();
    result.appendSlice(arg3) catch return 1;
    result.append('-') catch return 1;
    result.appendSlice(arg1) catch return 1;
    result.append('-') catch return 1;
    result.appendSlice(arg2) catch return 1;

    wasm_minimal_protocol_send_result_to_host(result.items.ptr, result.items.len);
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
