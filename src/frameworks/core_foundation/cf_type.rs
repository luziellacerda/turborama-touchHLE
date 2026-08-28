/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */
//! `CFType` (type-generic functions etc).

use super::{CFHashCode, CFIndex, CFTypeID};
use crate::dyld::{export_c_func, FunctionExports};
use crate::frameworks::foundation::NSUInteger;
use crate::objc::Class;
use crate::{msg, objc};
use crate::{msg_class, Environment};

pub type CFTypeRef = objc::id;

pub fn CFRetain(env: &mut Environment, object: CFTypeRef) -> CFTypeRef {
    assert!(!object.is_null()); // not allowed, unlike for normal objc objects
    objc::retain(env, object)
}
pub fn CFRelease(env: &mut Environment, object: CFTypeRef) {
    objc::release(env, object);
}

pub fn CFGetRetainCount(env: &mut Environment, object: CFTypeRef) -> CFIndex {
    let count: NSUInteger = msg![env; object retainCount];
    count as CFIndex
}

pub fn CFEqual(env: &mut Environment, object1: CFTypeRef, object2: CFTypeRef) -> bool {
    if object1 == object2 {
        return true;
    }
    // TODO: other classes
    let str_class: Class = msg_class![env; NSString class];
    let object1_class: Class = msg![env; object1 class];
    assert!(msg![env; object1_class isKindOfClass:str_class]);
    let object2_class: Class = msg![env; object2 class];
    assert!(msg![env; object2_class isKindOfClass:str_class]);
    // TODO: use isEqual: once it is fixed
    msg![env; object1 isEqualToString:object2]
}

pub fn CFHash(env: &mut Environment, object: CFTypeRef) -> CFHashCode {
    msg![env; object hash]
}

/// `CFTypeID` uniquely identifies an opaque CoreFoundation type. In touchHLE,
/// CF objects are their toll-free-bridged ObjC counterparts (a CFString is an
/// NSString, a CFArray is an NSArray, etc.). We use the *base* class pointer of
/// the CF type as the type ID. It's crucial to normalize to the base class
/// (e.g. NSArray) rather than the object's concrete class (e.g. some private
/// mutable-array subclass), because the per-type `CF...GetTypeID()` functions
/// return the base-class pointer, and `CFGetTypeID(x) == CFArrayGetTypeID()`
/// must hold.
pub fn CFGetTypeID(env: &mut Environment, object: CFTypeRef) -> CFTypeID {
    assert!(!object.is_null());
    let obj_class: Class = msg![env; object class];
    // Check against the known toll-free-bridged base classes and return the
    // base class pointer as the type ID.
    for base in ["NSString", "NSArray", "NSDictionary", "NSData", "NSNumber"] {
        let base_class: Class = env.objc.get_known_class(base, &mut env.mem);
        if msg![env; obj_class isKindOfClass:base_class] {
            return base_class.to_bits();
        }
    }
    // Fallback: use the object's own class pointer. Not matched by any
    // CF...GetTypeID(), but stable and unique per class.
    log!(
        "Warning: CFGetTypeID({:?}) on an unrecognized type, returning its class pointer",
        object
    );
    obj_class.to_bits()
}

/// Helper shared by the per-type `CF...GetTypeID()` functions: the type ID of a
/// CF type is the pointer of its toll-free-bridged base ObjC class.
pub fn type_id_for_class(env: &mut Environment, class_name: &str) -> CFTypeID {
    env.objc.get_known_class(class_name, &mut env.mem).to_bits()
}

pub const FUNCTIONS: FunctionExports = &[
    export_c_func!(CFRetain(_)),
    export_c_func!(CFRelease(_)),
    export_c_func!(CFGetRetainCount(_)),
    export_c_func!(CFEqual(_, _)),
    export_c_func!(CFHash(_)),
    export_c_func!(CFGetTypeID(_)),
];
