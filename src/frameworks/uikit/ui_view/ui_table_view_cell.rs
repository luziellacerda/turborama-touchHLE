/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */
//! `UITableViewCell`.

use crate::frameworks::core_graphics::CGRect;
use crate::frameworks::foundation::NSInteger;
use crate::objc::{
    id, impl_HostObject_with_superclass, msg, msg_class, msg_super, nil, objc_classes, release,
    retain, ClassExports, NSZonePtr,
};

pub struct UITableViewCellHostObject {
    superclass: super::UIViewHostObject,
    /// UILabel*, the default text label, created lazily.
    text_label: id,
    /// UIView*, the content view that holds the cell's custom content.
    content_view: id,
    /// NSString*, the reuse identifier this cell was created with.
    reuse_identifier: id,
}
impl_HostObject_with_superclass!(UITableViewCellHostObject);
impl Default for UITableViewCellHostObject {
    fn default() -> Self {
        UITableViewCellHostObject {
            superclass: Default::default(),
            text_label: nil,
            content_view: nil,
            reuse_identifier: nil,
        }
    }
}

pub const CLASSES: ClassExports = objc_classes! {

(env, this, _cmd);

@implementation UITableViewCell: UIView

+ (id)allocWithZone:(NSZonePtr)_zone {
    let host_object = Box::<UITableViewCellHostObject>::default();
    env.objc.alloc_object(this, host_object, &mut env.mem)
}

- (id)initWithStyle:(NSInteger)_style
    reuseIdentifier:(id)reuse_identifier { // NSString*
    let this: id = msg![env; this init];

    retain(env, reuse_identifier);
    env.objc.borrow_mut::<UITableViewCellHostObject>(this).reuse_identifier = reuse_identifier;

    // The content view is a plain subview filling the cell.
    let bounds: CGRect = msg![env; this bounds];
    let content_view: id = msg_class![env; UIView alloc];
    let content_view: id = msg![env; content_view initWithFrame:bounds];
    () = msg![env; this addSubview:content_view];
    env.objc.borrow_mut::<UITableViewCellHostObject>(this).content_view = content_view;

    this
}

- (id)initWithFrame:(CGRect)frame {
    let this: id = msg_super![env; this initWithFrame:frame];
    let content_view: id = msg_class![env; UIView alloc];
    let content_view: id = msg![env; content_view initWithFrame:frame];
    () = msg![env; this addSubview:content_view];
    env.objc.borrow_mut::<UITableViewCellHostObject>(this).content_view = content_view;
    this
}

- (())dealloc {
    let &UITableViewCellHostObject {
        text_label,
        content_view,
        reuse_identifier,
        ..
    } = env.objc.borrow::<UITableViewCellHostObject>(this);
    release(env, text_label);
    release(env, content_view);
    release(env, reuse_identifier);
    msg_super![env; this dealloc]
}

- (id)contentView {
    env.objc.borrow::<UITableViewCellHostObject>(this).content_view
}

- (id)reuseIdentifier {
    env.objc.borrow::<UITableViewCellHostObject>(this).reuse_identifier
}

// The default text label, created on first access (as on iOS).
- (id)textLabel {
    let existing = env.objc.borrow::<UITableViewCellHostObject>(this).text_label;
    if existing != nil {
        return existing;
    }
    let content_view = env.objc.borrow::<UITableViewCellHostObject>(this).content_view;
    let bounds: CGRect = msg![env; content_view bounds];
    let label: id = msg_class![env; UILabel alloc];
    let label: id = msg![env; label initWithFrame:bounds];
    () = msg![env; content_view addSubview:label];
    env.objc.borrow_mut::<UITableViewCellHostObject>(this).text_label = label;
    label
}

// Selection is a no-op for now (nothing is visually rendered differently).
- (())setSelected:(bool)_selected animated:(bool)_animated {
}
- (())setSelectionStyle:(NSInteger)_style {
}
- (())prepareForReuse {
}

@end

};
