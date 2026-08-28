/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */
//! `UIActionSheet`.
//!
//! Like [super::ui_alert_view], this is a stub: the sheet isn't actually
//! rendered, but it can be created, configured and "shown" so apps that use it
//! for optional menus keep running.

use crate::frameworks::core_graphics::CGRect;
use crate::frameworks::foundation::ns_string;
use crate::frameworks::foundation::NSInteger;
use crate::objc::{id, msg_super, nil, objc_classes, ClassExports};
use std::borrow::Cow;

pub const CLASSES: ClassExports = objc_classes! {

(env, this, _cmd);

@implementation UIActionSheet: UIView

- (id)initWithTitle:(id)title
                       delegate:(id)delegate
              cancelButtonTitle:(id)cancelButtonTitle
         destructiveButtonTitle:(id)destructiveButtonTitle
              otherButtonTitles:(id)otherButtonTitles {
    let title_str = if title == nil {
        Cow::from("(nil)")
    } else {
        ns_string::to_rust_string(env, title)
    };
    log!(
        "TODO: [(UIActionSheet*){:?} initWithTitle:{:?} delegate:{:?} cancelButtonTitle:{:?} destructiveButtonTitle:{:?} otherButtonTitles:{:?}]",
        this, title_str, delegate, cancelButtonTitle, destructiveButtonTitle, otherButtonTitles
    );
    msg_super![env; this init]
}

- (NSInteger)addButtonWithTitle:(id)title {
    log!(
        "TODO: [(UIActionSheet *){:?} addButtonWithTitle:{}]",
        this,
        ns_string::to_rust_string(env, title)
    );
    0
}

- (())showInView:(id)view {
    log!("TODO: [(UIActionSheet*){:?} showInView:{:?}]", this, view);
}

- (())showFromRect:(CGRect)rect inView:(id)view animated:(bool)_animated {
    log!(
        "TODO: [(UIActionSheet*){:?} showFromRect:{:?} inView:{:?}]",
        this, rect, view
    );
}

- (())showFromTabBar:(id)tab_bar {
    log!("TODO: [(UIActionSheet*){:?} showFromTabBar:{:?}]", this, tab_bar);
}

- (())showFromToolbar:(id)toolbar {
    log!("TODO: [(UIActionSheet*){:?} showFromToolbar:{:?}]", this, toolbar);
}

- (())dismissWithClickedButtonIndex:(NSInteger)_index animated:(bool)_animated {
    log!("TODO: [(UIActionSheet*){:?} dismissWithClickedButtonIndex:animated:]", this);
}

@end

};
