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

// Programmatic construction from an image (e.g. an icon from the app
// bundle), used among others by ad frameworks like Greystripe when
// building their navigation bars in code. Like the other initializers
// here, image, style, target and action are all ignored: touchHLE
// doesn't render bar button items or wire up their taps.
- (id)initWithImage:(id)_image // UIImage*
              style:(NSInteger)_style // UIBarButtonItemStyle
             target:(id)_target
             action:(SEL)_action {
    log!(
        "TODO: [(UIBarButtonItem*){:?} initWithImage:style:target:action:] (ignored)",
        this
    );
    this
}

// Wraps an arbitrary custom view as a bar button item. touchHLE doesn't
// actually place the view in the bar (UINavigationBar/UIToolbar are stubs
// here too), so the view is discarded; returning self just lets
// construction succeed.
- (id)initWithCustomView:(id)custom_view { // UIView*
    log!(
        "TODO: [(UIBarButtonItem*){:?} initWithCustomView:{:?}] (ignored)",
        this, custom_view
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

// UIBarButtonItem has various further properties which touchHLE doesn't
// use anywhere (there's no real toolbar/nav bar rendering), so setting
// them is safe to ignore.
- (())setEnabled:(bool)_enabled {
    log!("TODO: [(UIBarButtonItem*){:?} setEnabled:] (ignored)", this);
}
- (bool)isEnabled {
    true
}
- (())setStyle:(NSInteger)_style { // UIBarButtonItemStyle
    log!("TODO: [(UIBarButtonItem*){:?} setStyle:] (ignored)", this);
}
- (())setTag:(NSInteger)_tag {
    log!("TODO: [(UIBarButtonItem*){:?} setTag:] (ignored)", this);
}
- (())setTintColor:(id)_color { // UIColor*
    log!("TODO: [(UIBarButtonItem*){:?} setTintColor:] (ignored)", this);
}

@end

};
