/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */
//! `UIToolbar` and `UIBarButtonItem` (stubs).
//!
//! Some apps have a toolbar in their nib file without meaningfully using it.
//! These stubs let nib unarchiving succeed.

use crate::frameworks::foundation::NSInteger;
use crate::frameworks::uikit::ui_view::ui_control::UIControlEvents;
use crate::objc::{id, objc_classes, ClassExports, SEL};

pub const CLASSES: ClassExports = objc_classes! {

(env, this, _cmd);

@implementation UIToolbar: UIView

// initWithCoder: inherited from UIView handles nib decoding.

- (())setItems:(id)_items { // NSArray<UIBarButtonItem *> *
    log!("TODO: [(UIToolbar*){:?} setItems:] (ignored)", this);
}
- (())setItems:(id)_items animated:(bool)_animated {
    log!("TODO: [(UIToolbar*){:?} setItems:animated:] (ignored)", this);
}
- (())setBarStyle:(NSInteger)_style { // UIBarStyle
    log!("TODO: [(UIToolbar*){:?} setBarStyle:] (ignored)", this);
}

@end

@implementation UIBarButtonItem: NSObject

- (id)initWithCoder:(id)_coder {
    log!(
        "TODO: [(UIBarButtonItem*){:?} initWithCoder:] (keys ignored)",
        this
    );
    this
}

// Programmatic construction with one of the predefined system button styles
// (Done, Cancel, Add, etc.). The target/action are ignored here for the same
// reason addTarget:action:forControlEvents: below ignores them: touchHLE
// doesn't wire up bar button item taps to anything.
- (id)initWithBarButtonSystemItem:(NSInteger)_system_item // UIBarButtonSystemItem
                            target:(id)_target
                            action:(SEL)_action {
    log!(
        "TODO: [(UIBarButtonItem*){:?} initWithBarButtonSystemItem:target:action:] (ignored)",
        this
    );
    this
}

// Nib event connections send this UIControl message to bar button items.
- (())addTarget:(id)_target
         action:(SEL)_action
forControlEvents:(UIControlEvents)_events {
    log!(
        "TODO: [(UIBarButtonItem*){:?} addTarget:action:forControlEvents:] (ignored)",
        this
    );
}

@end

};
