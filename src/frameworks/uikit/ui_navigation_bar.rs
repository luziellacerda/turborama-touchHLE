/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */
//! `UINavigationBar` and `UINavigationItem` (stubs).
//!
//! Some apps have a navigation bar in their nib file (usually placed there
//! directly, rather than being managed by a `UINavigationController`)
//! without meaningfully using it. These stubs let nib unarchiving succeed
//! and let the most common navigation bar messages through without
//! crashing, matching the approach taken for `UIToolbar`.

use crate::frameworks::foundation::NSInteger;
use crate::objc::{id, msg, nil, objc_classes, ClassExports};

pub const CLASSES: ClassExports = objc_classes! {

(env, this, _cmd);

@implementation UINavigationBar: UIView

// initWithCoder: inherited from UIView handles nib decoding.

- (())setBarStyle:(NSInteger)_style { // UIBarStyle
    log!("TODO: [(UINavigationBar*){:?} setBarStyle:] (ignored)", this);
}
- (())setTintColor:(id)_color { // UIColor*
    log!("TODO: [(UINavigationBar*){:?} setTintColor:] (ignored)", this);
}
- (())setTranslucent:(bool)_translucent {
    log!("TODO: [(UINavigationBar*){:?} setTranslucent:] (ignored)", this);
}

- (())pushNavigationItem:(id)_item // UINavigationItem*
                 animated:(bool)_animated {
    log!("TODO: [(UINavigationBar*){:?} pushNavigationItem:animated:] (ignored)", this);
}
- (id)popNavigationItemAnimated:(bool)_animated {
    log!("TODO: [(UINavigationBar*){:?} popNavigationItemAnimated:] (ignored)", this);
    nil
}
- (())setItems:(id)_items { // NSArray<UINavigationItem*>*
    log!("TODO: [(UINavigationBar*){:?} setItems:] (ignored)", this);
}
- (())setItems:(id)_items animated:(bool)_animated {
    log!("TODO: [(UINavigationBar*){:?} setItems:animated:] (ignored)", this);
}
- (id)topItem {
    log!("TODO: [(UINavigationBar*){:?} topItem] (returning nil)", this);
    nil
}

@end

@implementation UINavigationItem: NSObject

- (id)initWithCoder:(id)_coder {
    log!(
        "TODO: [(UINavigationItem*){:?} initWithCoder:] (keys ignored)",
        this
    );
    this
}
- (id)initWithTitle:(id)_title { // NSString*
    log!(
        "TODO: [(UINavigationItem*){:?} initWithTitle:] (ignored)",
        this
    );
    this
}
- (())setTitle:(id)_title { // NSString*
    log!("TODO: [(UINavigationItem*){:?} setTitle:] (ignored)", this);
}
- (())setLeftBarButtonItem:(id)_item { // UIBarButtonItem*
    log!("TODO: [(UINavigationItem*){:?} setLeftBarButtonItem:] (ignored)", this);
}
- (())setRightBarButtonItem:(id)_item { // UIBarButtonItem*
    log!("TODO: [(UINavigationItem*){:?} setRightBarButtonItem:] (ignored)", this);
}
- (())setLeftBarButtonItem:(id)item animated:(bool)_animated { // UIBarButtonItem*
    () = msg![env; this setLeftBarButtonItem:item];
}
- (())setRightBarButtonItem:(id)item animated:(bool)_animated { // UIBarButtonItem*
    () = msg![env; this setRightBarButtonItem:item];
}
- (())setHidesBackButton:(bool)_hides {
}

@end

};
