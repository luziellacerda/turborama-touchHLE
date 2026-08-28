/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */
//! `CFStream` (`CFReadStream` / `CFWriteStream`).
//!
//! touchHLE has no real networking, so socket streams can't be created. These
//! stubs hand back null streams, which apps treat as "couldn't open a
//! connection" and fall back from, rather than crashing on the unimplemented
//! function.

use super::cf_allocator::CFAllocatorRef;
use super::cf_host::CFHostRef;
use super::CFTypeRef;
use crate::dyld::{export_c_func, FunctionExports};
use crate::mem::{MutPtr, Ptr};
use crate::Environment;

pub type CFReadStreamRef = CFTypeRef;
pub type CFWriteStreamRef = CFTypeRef;

fn CFStreamCreatePairWithSocketToCFHost(
    env: &mut Environment,
    _allocator: CFAllocatorRef,
    _host: CFHostRef,
    _port: i32,
    read_stream: MutPtr<CFReadStreamRef>,
    write_stream: MutPtr<CFWriteStreamRef>,
) {
    // No networking: hand back null streams so the app sees the connection
    // couldn't be opened.
    if !read_stream.is_null() {
        env.mem.write(read_stream, Ptr::null());
    }
    if !write_stream.is_null() {
        env.mem.write(write_stream, Ptr::null());
    }
}

pub const FUNCTIONS: FunctionExports =
    &[export_c_func!(CFStreamCreatePairWithSocketToCFHost(_, _, _, _, _))];
