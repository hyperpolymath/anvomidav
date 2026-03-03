// ANVOMIDAV — Zig FFI Implementation
//
// This module implements the binary interface declared in the Idris ABI.
// It uses standard C conventions to ensure compatibility with both Idris 
// and Rust consumers.
//
// SAFETY: All memory allocated in this layer MUST be explicitly freed 
// via the `anvomidav_free*` functions.

const std = @import("std");

const VERSION = "0.1.0";
const BUILD_INFO = "Anvomidav (Zig) - Verified Verification Core";

/// THREAD-LOCAL ERRORS: Stores the last error message for FFI retrieval.
threadlocal var last_error: ?[]const u8 = null;

//==============================================================================
// CORE TYPES: ABI-Stable Representations
//==============================================================================

/// RESULT CODES: Must match the `Result` type in `ABI/Types.idr`.
pub const Result = enum(c_int) {
    ok = 0,
    @"error" = 1,
    invalid_param = 2,
    out_of_memory = 3,
    null_pointer = 4,
};

/// OPAQUE HANDLE: Prevents external languages from inspecting internal state.
pub const Handle = opaque {
    allocator: std.mem.Allocator,
    initialized: bool,
};

//==============================================================================
// LIFECYCLE: Resource Management
//==============================================================================

/// INITIALIZATION: Allocates the opaque handle on the C heap.
export fn anvomidav_init() ?*Handle {
    const allocator = std.heap.c_allocator;

    const handle = allocator.create(Handle) catch {
        last_error = "FFI: Out of memory";
        return null;
    };

    handle.* = .{
        .allocator = allocator,
        .initialized = true,
    };

    return handle;
}

/// CLEANUP: Safely destroys the handle.
export fn anvomidav_free(handle: ?*Handle) void {
    const h = handle orelse return;
    const allocator = h.allocator;
    h.initialized = false;
    allocator.destroy(h);
}
