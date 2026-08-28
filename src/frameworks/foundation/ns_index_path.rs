/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */
//! `NSIndexPath`.
//!
//! Only the two-index form used by UITableView (section + row) is implemented.

use crate::frameworks::foundation::NSUInteger;
use crate::objc::{autorelease, id, msg, objc_classes, ClassExports, HostObject, NSZonePtr};

#[derive(Default)]
struct NSIndexPathHostObject {
    /// The indexes making up this path. For UITableView this is [section, row].
    indexes: Vec<NSUInteger>,
}
impl HostObject for NSIndexPathHostObject {}

pub const CLASSES: ClassExports = objc_classes! {

(env, this, _cmd);

@implementation NSIndexPath: NSObject

+ (id)allocWithZone:(NSZonePtr)_zone {
    let host_object = Box::<NSIndexPathHostObject>::default();
    env.objc.alloc_object(this, host_object, &mut env.mem)
}

+ (id)indexPathForRow:(NSUInteger)row inSection:(NSUInteger)section {
    let new: id = msg![env; this alloc];
    env.objc.borrow_mut::<NSIndexPathHostObject>(new).indexes = vec![section, row];
    autorelease(env, new)
}

+ (id)indexPathWithIndex:(NSUInteger)index {
    let new: id = msg![env; this alloc];
    env.objc.borrow_mut::<NSIndexPathHostObject>(new).indexes = vec![index];
    autorelease(env, new)
}

- (NSUInteger)section {
    // First index is the section (see indexPathForRow:inSection:).
    env.objc
        .borrow::<NSIndexPathHostObject>(this)
        .indexes
        .first()
        .copied()
        .unwrap_or(0)
}

- (NSUInteger)row {
    // Second index is the row.
    env.objc
        .borrow::<NSIndexPathHostObject>(this)
        .indexes
        .get(1)
        .copied()
        .unwrap_or(0)
}

- (NSUInteger)indexAtPosition:(NSUInteger)position {
    env.objc
        .borrow::<NSIndexPathHostObject>(this)
        .indexes
        .get(position as usize)
        .copied()
        .unwrap_or(0)
}

- (NSUInteger)length {
    env.objc.borrow::<NSIndexPathHostObject>(this).indexes.len() as NSUInteger
}

@end

};
