/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */
//! `UIPickerView`.

use crate::objc::{
    id, impl_HostObject_with_superclass, nil, objc_classes, todo_objc_setter, ClassExports,
    NSZonePtr,
};

// TODO: rendering

struct UIPickerViewHostObject {
    superclass: super::UIViewHostObject,
    /// `id<UIPickerViewDelegate>`, weak reference
    delegate: id,
    /// `id<UIPickerViewDataSource>`, weak reference
    data_source: id,
}
impl_HostObject_with_superclass!(UIPickerViewHostObject);
impl Default for UIPickerViewHostObject {
    fn default() -> Self {
        UIPickerViewHostObject {
            superclass: Default::default(),
            delegate: nil,
            data_source: nil,
        }
    }
}

pub const CLASSES: ClassExports = objc_classes! {

(env, this, _cmd);

@implementation UIPickerView: UIView

+ (id)allocWithZone:(NSZonePtr)_zone {
    let host_object = Box::<UIPickerViewHostObject>::default();
    env.objc.alloc_object(this, host_object, &mut env.mem)
}

- (())setShowsSelectionIndicator:(bool)shows {
    todo_objc_setter!(this, shows);
}

// Both of these are weak references in UIKit, so they are stored without
// retaining. Nothing reads them yet - the view isn't rendered - but nibs
// connect them, and refusing the connection aborts the app.
- (())setDelegate:(id)delegate {
    env.objc.borrow_mut::<UIPickerViewHostObject>(this).delegate = delegate;
}
- (id)delegate {
    env.objc.borrow::<UIPickerViewHostObject>(this).delegate
}
- (())setDataSource:(id)data_source {
    env.objc.borrow_mut::<UIPickerViewHostObject>(this).data_source = data_source;
}
- (id)dataSource {
    env.objc.borrow::<UIPickerViewHostObject>(this).data_source
}

- (())reloadAllComponents {
    // TODO: rendering, so nothing to reload yet.
}
- (())reloadComponent:(i32)_component {
    // TODO: rendering, so nothing to reload yet.
}

@end

};
