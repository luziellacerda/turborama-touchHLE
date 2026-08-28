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
use crate::objc::{id, msg, msg_super, nil, objc_classes, ClassExports};
use std::borrow::Cow;
use std::collections::HashMap;

/// Delegates and button titles of action sheets, keyed by sheet. Not retaining.
#[derive(Default)]
pub struct State {
    delegates: HashMap<id, id>,
    /// Button titles in index order. iOS orders them: destructive first (if
    /// any), then the other buttons, then cancel last.
    buttons: HashMap<id, Vec<id>>,
}
impl State {
    fn get(env: &mut crate::Environment) -> &mut Self {
        &mut env.framework_state.uikit.ui_view.ui_action_sheet
    }
}

/// The sheet can't be displayed, so it can't be answered by the user either.
/// Report the first button to the delegate so apps that block until a choice is
/// made keep running, rather than waiting forever.
fn auto_select_first_button(env: &mut crate::Environment, sheet: id) {
    let delegate = State::get(env).delegates.get(&sheet).copied().unwrap_or(nil);
    if delegate == nil {
        return;
    }
    let index: NSInteger = 0;

    // Follow the order iOS uses: present first (apps often set up their state
    // in didPresentActionSheet:), then the button click, then the dismissal.
    if delegate_responds(env, delegate, "willPresentActionSheet:") {
        () = msg![env; delegate willPresentActionSheet:sheet];
    }
    if delegate_responds(env, delegate, "didPresentActionSheet:") {
        () = msg![env; delegate didPresentActionSheet:sheet];
    }
    if delegate_responds(env, delegate, "actionSheet:clickedButtonAtIndex:") {
        () = msg![env; delegate actionSheet:sheet clickedButtonAtIndex:index];
    }
    if delegate_responds(env, delegate, "actionSheet:willDismissWithButtonIndex:") {
        () = msg![env; delegate actionSheet:sheet willDismissWithButtonIndex:index];
    }
    if delegate_responds(env, delegate, "actionSheet:didDismissWithButtonIndex:") {
        () = msg![env; delegate actionSheet:sheet didDismissWithButtonIndex:index];
    }
}

fn delegate_responds(env: &mut crate::Environment, delegate: id, selector_name: &str) -> bool {
    match env.objc.lookup_selector(selector_name) {
        Some(sel) => msg![env; delegate respondsToSelector:sel],
        None => false,
    }
}

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
    let new: id = msg_super![env; this init];
    State::get(env).delegates.insert(new, delegate);
    // Record the buttons in the order iOS assigns indices: destructive first,
    // then the other buttons, then cancel last. Only the first entry of
    // otherButtonTitles is recorded (the rest is a nil-terminated vararg list,
    // which isn't modelled here).
    let mut buttons: Vec<id> = Vec::new();
    if destructiveButtonTitle != nil {
        buttons.push(destructiveButtonTitle);
    }
    if otherButtonTitles != nil {
        buttons.push(otherButtonTitles);
    }
    if cancelButtonTitle != nil {
        buttons.push(cancelButtonTitle);
    }
    State::get(env).buttons.insert(new, buttons);
    new
}

- (NSInteger)addButtonWithTitle:(id)title {
    log!(
        "TODO: [(UIActionSheet *){:?} addButtonWithTitle:{}]",
        this,
        ns_string::to_rust_string(env, title)
    );
    let buttons = State::get(env).buttons.entry(this).or_default();
    buttons.push(title);
    (buttons.len() - 1) as NSInteger
}

- (NSInteger)numberOfButtons {
    State::get(env)
        .buttons
        .get(&this)
        .map_or(0, |buttons| buttons.len()) as NSInteger
}

- (id)buttonTitleAtIndex:(NSInteger)index {
    if index < 0 {
        return nil;
    }
    State::get(env)
        .buttons
        .get(&this)
        .and_then(|buttons| buttons.get(index as usize).copied())
        .unwrap_or(nil)
}

- (())showInView:(id)view {
    log!("TODO: [(UIActionSheet*){:?} showInView:{:?}]", this, view);
    auto_select_first_button(env, this);
}

- (())showFromRect:(CGRect)rect inView:(id)view animated:(bool)_animated {
    log!(
        "TODO: [(UIActionSheet*){:?} showFromRect:{:?} inView:{:?}]",
        this, rect, view
    );
    auto_select_first_button(env, this);
}

- (())showFromTabBar:(id)tab_bar {
    log!("TODO: [(UIActionSheet*){:?} showFromTabBar:{:?}]", this, tab_bar);
    auto_select_first_button(env, this);
}

- (())showFromToolbar:(id)toolbar {
    log!("TODO: [(UIActionSheet*){:?} showFromToolbar:{:?}]", this, toolbar);
    auto_select_first_button(env, this);
}

- (())dismissWithClickedButtonIndex:(NSInteger)_index animated:(bool)_animated {
    log!("TODO: [(UIActionSheet*){:?} dismissWithClickedButtonIndex:animated:]", this);
}

- (())setActionSheetStyle:(NSInteger)_style { // UIActionSheetStyle
    // The sheet isn't rendered by touchHLE, so its style has no effect.
}
- (NSInteger)actionSheetStyle {
    0 // UIActionSheetStyleAutomatic
}
- (())setDelegate:(id)delegate {
    State::get(env).delegates.insert(this, delegate);
}
- (id)delegate {
    State::get(env).delegates.get(&this).copied().unwrap_or(nil)
}
- (())setTitle:(id)_title {
}
- (NSInteger)cancelButtonIndex {
    // init puts the cancel button last.
    let count = State::get(env)
        .buttons
        .get(&this)
        .map_or(0, |buttons| buttons.len());
    if count == 0 {
        -1
    } else {
        (count - 1) as NSInteger
    }
}
- (())setCancelButtonIndex:(NSInteger)_index {
}
- (bool)isVisible {
    false
}

@end

};
