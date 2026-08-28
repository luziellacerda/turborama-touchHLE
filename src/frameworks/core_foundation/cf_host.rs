/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */
//! `CFHost`.
//!
//! touchHLE has no real networking, so this is a stub: a host handle can be
//! created and configured, but name resolution always fails. Apps that use
//! CFHost for optional online features (leaderboards, ads, etc.) will take
//! their "couldn't reach the server" path and keep running, rather than
//! crashing on the unimplemented functions.

use super::cf_allocator::{kCFAllocatorDefault, CFAllocatorRef};
use super::cf_string::CFStringRef;
use super::CFTypeRef;
use crate::dyld::{export_c_func, FunctionExports};
use crate::mem::{MutVoidPtr, Ptr};
use crate::objc::msg_class;
use crate::Environment;

pub type CFHostRef = CFTypeRef;
type CFHostInfoType = u32;

fn CFHostCreateWithName(
    env: &mut Environment,
    allocator: CFAllocatorRef,
    _hostname: CFStringRef,
) -> CFHostRef {
    assert!(allocator == kCFAllocatorDefault || env.mem.read(allocator).is_system_default());
    // Return an opaque handle the app can configure but that resolves to
    // nothing. A plain object is enough since we never read anything back.
    msg_class![env; NSObject new]
}

fn CFHostSetClient(
    _env: &mut Environment,
    _host: CFHostRef,
    _callback: MutVoidPtr,
    _context: MutVoidPtr,
) -> bool {
    // Accept and ignore the client callback registration.
    true
}

fn CFHostScheduleWithRunLoop(
    _env: &mut Environment,
    _host: CFHostRef,
    _run_loop: CFTypeRef,
    _run_loop_mode: CFStringRef,
) {
    // no-op
}

fn CFHostUnscheduleFromRunLoop(
    _env: &mut Environment,
    _host: CFHostRef,
    _run_loop: CFTypeRef,
    _run_loop_mode: CFStringRef,
) {
    // no-op
}

fn CFHostStartInfoResolution(
    env: &mut Environment,
    _host: CFHostRef,
    _info: CFHostInfoType,
    error: MutVoidPtr,
) -> bool {
    // There's no real network, so resolution fails. Zero out the CFStreamError
    // if the app provided one (all-zero means "no error struct filled in", and
    // the false return already signals failure).
    if !error.is_null() {
        // CFStreamError is { CFIndex domain; SInt32 error; } == 8 bytes.
        env.mem.write(error.cast::<u32>(), 0);
        env.mem.write((error + 4).cast::<u32>(), 0);
    }
    false
}

fn CFHostGetAddressing(
    _env: &mut Environment,
    _host: CFHostRef,
    has_been_resolved: MutVoidPtr,
) -> CFTypeRef {
    // Nothing was resolved.
    if !has_been_resolved.is_null() {
        _env.mem.write(has_been_resolved.cast::<u8>(), 0); // false
    }
    Ptr::null()
}

fn CFHostCancelInfoResolution(
    _env: &mut Environment,
    _host: CFHostRef,
    _info: CFHostInfoType,
) {
    // no-op
}

pub const FUNCTIONS: FunctionExports = &[
    export_c_func!(CFHostCreateWithName(_, _)),
    export_c_func!(CFHostSetClient(_, _, _)),
    export_c_func!(CFHostScheduleWithRunLoop(_, _, _)),
    export_c_func!(CFHostUnscheduleFromRunLoop(_, _, _)),
    export_c_func!(CFHostStartInfoResolution(_, _, _)),
    export_c_func!(CFHostGetAddressing(_, _)),
    export_c_func!(CFHostCancelInfoResolution(_, _)),
];
