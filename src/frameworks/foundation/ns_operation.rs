/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */
//! `NSOperation` and `NSOperationQueue`.
//!
//! touchHLE runs operations synchronously on the calling thread, which is
//! sufficient for the ways games typically use these classes (kick off a bit of
//! work and rely on it completing).

use crate::objc::{
    autorelease, id, msg, msg_class, objc_classes, ClassExports, HostObject, NSZonePtr,
};

struct NSOperationHostObject {
    // The operation's completion block / target-action isn't modelled; concrete
    // subclasses (e.g. NSInvocationOperation) override main.
    cancelled: bool,
}
impl HostObject for NSOperationHostObject {}

struct NSOperationQueueHostObject {
    operations: id, // NSMutableArray
    suspended: bool,
    max_concurrent: i32,
}
impl HostObject for NSOperationQueueHostObject {}

pub const CLASSES: ClassExports = objc_classes! {

(env, this, _cmd);

@implementation NSOperation: NSObject

+ (id)allocWithZone:(NSZonePtr)_zone {
    let host_object = Box::new(NSOperationHostObject { cancelled: false });
    env.objc.alloc_object(this, host_object, &mut env.mem)
}

- (())start {
    // Default implementation runs main synchronously unless cancelled.
    let cancelled = env.objc.borrow::<NSOperationHostObject>(this).cancelled;
    if !cancelled {
        () = msg![env; this main];
    }
}
- (())main {
    // Overridden by concrete subclasses.
}
- (())cancel {
    env.objc.borrow_mut::<NSOperationHostObject>(this).cancelled = true;
}
- (bool)isCancelled {
    env.objc.borrow::<NSOperationHostObject>(this).cancelled
}
- (bool)isFinished {
    true
}
- (bool)isExecuting {
    false
}
- (bool)isConcurrent {
    false
}
- (())setCompletionBlock:(id)_block {
}
- (())setQueuePriority:(i32)_priority {
}
- (())addDependency:(id)_op {
}
- (())removeDependency:(id)_op {
}

@end

@implementation NSOperationQueue: NSObject

+ (id)allocWithZone:(NSZonePtr)_zone {
    let operations: id = msg_class![env; NSMutableArray new];
    let host_object = Box::new(NSOperationQueueHostObject {
        operations,
        suspended: false,
        max_concurrent: -1,
    });
    env.objc.alloc_object(this, host_object, &mut env.mem)
}

+ (id)mainQueue {
    // Return a lazily-created shared queue.
    let queue: id = msg![env; this new];
    autorelease(env, queue)
}

- (())addOperation:(id)op {
    // Run synchronously: start the operation immediately, then record it.
    () = msg![env; op start];
    let operations = env.objc.borrow::<NSOperationQueueHostObject>(this).operations;
    () = msg![env; operations addObject:op];
}
- (())addOperationWithBlock:(id)_block {
    // touchHLE doesn't support calling blocks, so this can't run the work.
    log!("TODO: [NSOperationQueue addOperationWithBlock:] (ignored)");
}
- (id)operations {
    env.objc.borrow::<NSOperationQueueHostObject>(this).operations
}
- (())setMaxConcurrentOperationCount:(i32)count {
    env.objc.borrow_mut::<NSOperationQueueHostObject>(this).max_concurrent = count;
}
- (i32)maxConcurrentOperationCount {
    env.objc.borrow::<NSOperationQueueHostObject>(this).max_concurrent
}
- (())setSuspended:(bool)suspended {
    env.objc.borrow_mut::<NSOperationQueueHostObject>(this).suspended = suspended;
}
- (bool)isSuspended {
    env.objc.borrow::<NSOperationQueueHostObject>(this).suspended
}
- (())cancelAllOperations {
    let operations = env.objc.borrow::<NSOperationQueueHostObject>(this).operations;
    let count: crate::frameworks::foundation::NSUInteger = msg![env; operations count];
    for i in 0..count {
        let op: id = msg![env; operations objectAtIndex:i];
        () = msg![env; op cancel];
    }
}
- (())waitUntilAllOperationsAreFinished {
    // Operations run synchronously in addOperation:, so nothing to wait for.
}

- (())dealloc {
    let operations = env.objc.borrow::<NSOperationQueueHostObject>(this).operations;
    () = msg![env; operations release];
    env.objc.dealloc_object(this, &mut env.mem)
}

@end

};
