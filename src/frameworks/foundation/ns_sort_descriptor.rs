/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */
//! `NSSortDescriptor`.

use super::{ns_string, NSComparisonResult, NSOrderedSame};
use crate::objc::{
    autorelease, id, msg, msg_send, nil, objc_classes, release, retain, ClassExports, HostObject,
    NSZonePtr,
};
use crate::Environment;

#[derive(Default)]
struct NSSortDescriptorHostObject {
    /// Retained.
    key: id,
    ascending: bool,
}
impl HostObject for NSSortDescriptorHostObject {}

pub const CLASSES: ClassExports = objc_classes! {

(env, this, _cmd);

@implementation NSSortDescriptor: NSObject

+ (id)allocWithZone:(NSZonePtr)_zone {
    let host_object = Box::<NSSortDescriptorHostObject>::default();
    env.objc.alloc_object(this, host_object, &mut env.mem)
}

+ (id)sortDescriptorWithKey:(id)key // NSString*
                   ascending:(bool)ascending {
    let new: id = msg![env; this alloc];
    let new: id = msg![env; new initWithKey:key ascending:ascending];
    autorelease(env, new)
}

- (id)initWithKey:(id)key // NSString*
        ascending:(bool)ascending {
    retain(env, key);
    let host_object = NSSortDescriptorHostObject { key, ascending };
    *env.objc.borrow_mut(this) = host_object;
    this
}

- (id)key {
    env.objc.borrow::<NSSortDescriptorHostObject>(this).key
}
- (bool)ascending {
    env.objc.borrow::<NSSortDescriptorHostObject>(this).ascending
}

// This only covers the common case: `key` is treated as the name of a
// zero-argument getter method on the compared objects (e.g. a simple
// `@property`), and the returned values are compared with `compare:`
// (which NSString and NSNumber both implement). Real NSSortDescriptor
// supports full KVC key paths (dotted paths, private ivar fallback, custom
// selectors/comparators via other initializers), none of which touchHLE
// implements; objects that don't respond to the key or to `compare:` are
// treated as equal for this descriptor rather than causing a crash.
- (NSComparisonResult)compareObject:(id)object1 toObject:(id)object2 {
    let &NSSortDescriptorHostObject { key, ascending } = env.objc.borrow(this);
    if key == nil {
        return NSOrderedSame;
    }
    let key_string = ns_string::to_rust_string(env, key).to_string();
    let getter = env.objc.register_host_selector(key_string, &mut env.mem);

    let value1: id = if object1 != nil && env.objc.object_has_method(&env.mem, object1, getter) {
        msg_send(env, (object1, getter))
    } else {
        nil
    };
    let value2: id = if object2 != nil && env.objc.object_has_method(&env.mem, object2, getter) {
        msg_send(env, (object2, getter))
    } else {
        nil
    };

    let compare_sel = env
        .objc
        .register_host_selector("compare:".to_string(), &mut env.mem);
    let result: NSComparisonResult =
        if value1 != nil && env.objc.object_has_method(&env.mem, value1, compare_sel) {
            msg_send(env, (value1, compare_sel, value2))
        } else {
            NSOrderedSame
        };

    if ascending {
        result
    } else {
        -result
    }
}

- (())dealloc {
    let &NSSortDescriptorHostObject { key, .. } = env.objc.borrow(this);
    release(env, key);
    env.objc.dealloc_object(this, &mut env.mem)
}

@end

};
