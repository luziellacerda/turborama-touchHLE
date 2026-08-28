/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */
//! `NSStream` and `NSInputStream`.
//!
//! `NSInputStream` is implemented on top of `NSData`: when the stream is
//! opened, the whole file is read into an `NSData`, and `read:maxLength:`
//! serves bytes from that buffer via a running offset. This covers the common
//! "open a file, read it sequentially, close it" usage without a real
//! incremental I/O pipeline.

use crate::frameworks::foundation::{NSInteger, NSRange, NSUInteger};
use crate::mem::{ConstVoidPtr, MutPtr};
use crate::objc::{
    autorelease, id, msg, msg_class, nil, objc_classes, release, retain, ClassExports, HostObject,
    NSZonePtr,
};

/// Values of the `NSStreamStatus` enum that we use.
const NSStreamStatusNotOpen: NSInteger = 0;
const NSStreamStatusOpen: NSInteger = 2;
const NSStreamStatusAtEnd: NSInteger = 5;
const NSStreamStatusClosed: NSInteger = 6;

#[derive(Default)]
struct NSInputStreamHostObject {
    /// Path the stream was created with (retained NSString), or nil.
    path: id,
    /// Backing data once opened (retained NSData), or nil.
    data: id,
    /// Current read offset into `data`.
    offset: NSUInteger,
    status: NSInteger,
}
impl HostObject for NSInputStreamHostObject {}

pub const CLASSES: ClassExports = objc_classes! {

(env, this, _cmd);

@implementation NSStream: NSObject
@end

@implementation NSInputStream: NSStream

+ (id)allocWithZone:(NSZonePtr)_zone {
    let host_object = Box::new(NSInputStreamHostObject {
        status: NSStreamStatusNotOpen,
        ..Default::default()
    });
    env.objc.alloc_object(this, host_object, &mut env.mem)
}

+ (id)inputStreamWithFileAtPath:(id)path { // NSString*
    let new: id = msg![env; this alloc];
    let new: id = msg![env; new initWithFileAtPath:path];
    autorelease(env, new)
}

- (id)initWithFileAtPath:(id)path { // NSString*
    retain(env, path);
    env.objc.borrow_mut::<NSInputStreamHostObject>(this).path = path;
    this
}

- (())dealloc {
    let &NSInputStreamHostObject { path, data, .. } =
        env.objc.borrow::<NSInputStreamHostObject>(this);
    release(env, path);
    release(env, data);
    env.objc.dealloc_object(this, &mut env.mem)
}

- (())open {
    let path = env.objc.borrow::<NSInputStreamHostObject>(this).path;
    // Read the whole file up front; a nil result leaves data == nil, which
    // read:maxLength: treats as an empty/at-end stream.
    let data: id = if path == nil {
        nil
    } else {
        msg_class![env; NSData dataWithContentsOfFile:path]
    };
    retain(env, data);
    let host = env.objc.borrow_mut::<NSInputStreamHostObject>(this);
    host.data = data;
    host.offset = 0;
    host.status = NSStreamStatusOpen;
}

- (())close {
    env.objc.borrow_mut::<NSInputStreamHostObject>(this).status = NSStreamStatusClosed;
}

- (NSInteger)streamStatus {
    env.objc.borrow::<NSInputStreamHostObject>(this).status
}

- (bool)hasBytesAvailable {
    let &NSInputStreamHostObject { data, offset, .. } =
        env.objc.borrow::<NSInputStreamHostObject>(this);
    if data == nil {
        return false;
    }
    let length: NSUInteger = msg![env; data length];
    offset < length
}

- (NSInteger)read:(MutPtr<u8>)buffer maxLength:(NSUInteger)max_length {
    let &NSInputStreamHostObject { data, offset, .. } =
        env.objc.borrow::<NSInputStreamHostObject>(this);
    if data == nil {
        return 0;
    }
    let length: NSUInteger = msg![env; data length];
    let remaining = length.saturating_sub(offset);
    let to_read = remaining.min(max_length);
    if to_read == 0 {
        env.objc.borrow_mut::<NSInputStreamHostObject>(this).status = NSStreamStatusAtEnd;
        return 0;
    }

    // Copy the next `to_read` bytes out of the NSData buffer into the guest
    // buffer via NSData's getBytes:range:.
    let range = NSRange {
        location: offset,
        length: to_read,
    };
    () = msg![env; data getBytes:buffer range:range];

    env.objc.borrow_mut::<NSInputStreamHostObject>(this).offset = offset + to_read;
    to_read.try_into().unwrap()
}

- (bool)getBuffer:(MutPtr<ConstVoidPtr>)_buffer length:(MutPtr<NSUInteger>)_length {
    // Not supported; callers must fall back to read:maxLength:.
    false
}

// Delegate / run-loop scheduling: accepted but unused, since reads are
// synchronous here.
- (())setDelegate:(id)_delegate {
}
- (id)delegate {
    nil
}
- (())scheduleInRunLoop:(id)_run_loop forMode:(id)_mode {
}
- (())removeFromRunLoop:(id)_run_loop forMode:(id)_mode {
}

@end

};
