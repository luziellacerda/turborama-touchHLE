/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */
//! `UITabBar` and `UITabBarItem`.
//!
//! Simplified stubs: the tab bar isn't rendered, but it can be created from a
//! NIB or in code, and it tracks its items and selection so apps that query
//! them keep working.

use crate::frameworks::foundation::NSInteger;
use crate::objc::{
    autorelease, id, impl_HostObject_with_superclass, msg, msg_class, msg_super, nil, objc_classes,
    release, retain, ClassExports, HostObject, NSZonePtr,
};

pub struct UITabBarHostObject {
    superclass: super::UIViewHostObject,
    /// NSArray* of UITabBarItem*, retained.
    items: id,
    /// UITabBarItem*, the selected item (non-retaining).
    selected_item: id,
    /// Delegate (non-retaining).
    delegate: id,
}
impl_HostObject_with_superclass!(UITabBarHostObject);
impl Default for UITabBarHostObject {
    fn default() -> Self {
        UITabBarHostObject {
            superclass: Default::default(),
            items: nil,
            selected_item: nil,
            delegate: nil,
        }
    }
}

#[derive(Default)]
struct UITabBarItemHostObject {
    /// NSString*, retained.
    title: id,
    /// UIImage*, retained.
    image: id,
    tag: NSInteger,
}
impl HostObject for UITabBarItemHostObject {}

pub const CLASSES: ClassExports = objc_classes! {

(env, this, _cmd);

@implementation UITabBar: UIView

+ (id)allocWithZone:(NSZonePtr)_zone {
    let host_object = Box::<UITabBarHostObject>::default();
    env.objc.alloc_object(this, host_object, &mut env.mem)
}

- (id)initWithCoder:(id)coder {
    // The tab bar isn't rendered; just decode the standard view geometry.
    msg_super![env; this initWithCoder:coder]
}

- (())setItems:(id)items { // NSArray*
    let old = env.objc.borrow::<UITabBarHostObject>(this).items;
    retain(env, items);
    release(env, old);
    env.objc.borrow_mut::<UITabBarHostObject>(this).items = items;
}
- (())setItems:(id)items animated:(bool)_animated { // NSArray*
    () = msg![env; this setItems:items];
}
- (id)items {
    env.objc.borrow::<UITabBarHostObject>(this).items
}

- (())setSelectedItem:(id)selected_item { // UITabBarItem*
    env.objc.borrow_mut::<UITabBarHostObject>(this).selected_item = selected_item;
}
- (id)selectedItem {
    env.objc.borrow::<UITabBarHostObject>(this).selected_item
}

- (())setDelegate:(id)delegate {
    env.objc.borrow_mut::<UITabBarHostObject>(this).delegate = delegate;
}
- (id)delegate {
    env.objc.borrow::<UITabBarHostObject>(this).delegate
}

- (())dealloc {
    let items = env.objc.borrow::<UITabBarHostObject>(this).items;
    release(env, items);
    msg_super![env; this dealloc]
}

@end

@implementation UITabBarItem: NSObject

+ (id)allocWithZone:(NSZonePtr)_zone {
    let host_object = Box::<UITabBarItemHostObject>::default();
    env.objc.alloc_object(this, host_object, &mut env.mem)
}

- (id)initWithTitle:(id)title // NSString*
              image:(id)image // UIImage*
                tag:(NSInteger)tag {
    let this: id = msg![env; this init];
    retain(env, title);
    retain(env, image);
    let host = env.objc.borrow_mut::<UITabBarItemHostObject>(this);
    host.title = title;
    host.image = image;
    host.tag = tag;
    this
}

- (id)initWithCoder:(id)_coder {
    msg![env; this init]
}

- (())setTitle:(id)title { // NSString*
    let old = env.objc.borrow::<UITabBarItemHostObject>(this).title;
    retain(env, title);
    release(env, old);
    env.objc.borrow_mut::<UITabBarItemHostObject>(this).title = title;
}
- (id)title {
    env.objc.borrow::<UITabBarItemHostObject>(this).title
}

- (())setImage:(id)image { // UIImage*
    let old = env.objc.borrow::<UITabBarItemHostObject>(this).image;
    retain(env, image);
    release(env, old);
    env.objc.borrow_mut::<UITabBarItemHostObject>(this).image = image;
}
- (id)image {
    env.objc.borrow::<UITabBarItemHostObject>(this).image
}

- (())setTag:(NSInteger)tag {
    env.objc.borrow_mut::<UITabBarItemHostObject>(this).tag = tag;
}
- (NSInteger)tag {
    env.objc.borrow::<UITabBarItemHostObject>(this).tag
}

- (())dealloc {
    let &UITabBarItemHostObject { title, image, .. } =
        env.objc.borrow::<UITabBarItemHostObject>(this);
    release(env, title);
    release(env, image);
    env.objc.dealloc_object(this, &mut env.mem)
}

@end

};
