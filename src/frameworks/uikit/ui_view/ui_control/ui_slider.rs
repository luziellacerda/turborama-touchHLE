/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */
//! `UISlider`.

use super::UIControlState;
use crate::frameworks::core_graphics::CGRect;
use crate::objc::{
    id, impl_HostObject_with_superclass, msg_super, nil, objc_classes, todo_objc_setter,
    ClassExports, NSZonePtr,
};

pub struct UISliderHostObject {
    superclass: super::UIControlHostObject,
    value: f32,
    minimum_value: f32,
    maximum_value: f32,
    continuous: bool,
}
impl_HostObject_with_superclass!(UISliderHostObject);
impl Default for UISliderHostObject {
    fn default() -> Self {
        UISliderHostObject {
            superclass: Default::default(),
            value: 0.0,
            minimum_value: 0.0,
            maximum_value: 1.0,
            // UIKit's default is YES.
            continuous: true,
        }
    }
}

pub const CLASSES: ClassExports = objc_classes! {

(env, this, _cmd);

@implementation UISlider: UIControl

+ (id)allocWithZone:(NSZonePtr)_zone {
    let host_object = Box::<UISliderHostObject>::default();
    env.objc.alloc_object(this, host_object, &mut env.mem)
}

- (id)initWithFrame:(CGRect)frame {
    log!("[(UISlider*){:?} initWithFrame:{:?}] TODO: Implement UISlider. The control won't be rendered.", this, frame);
    msg_super![env; this initWithFrame:frame]
}

// NSCoding implementation
- (id)initWithCoder:(id)coder {
    log!("[(UISlider*){:?} initWithCoder:{:?}] TODO: Implement UISlider. The control won't be rendered.", this, coder);
    msg_super![env; this initWithCoder:coder]
}

- (())setMinimumValueImage:(id)img { // UIImage *
    todo_objc_setter!(this, img);
}
- (())setMaximumValueImage:(id)img { // UIImage *
    todo_objc_setter!(this, img);
}

// State-dependent images. The slider isn't rendered, so these have no effect,
// but apps set them while configuring the control.
- (())setThumbImage:(id)_img // UIImage *
           forState:(UIControlState)_state {
}
- (())setMinimumTrackImage:(id)_img // UIImage *
                  forState:(UIControlState)_state {
}
- (())setMaximumTrackImage:(id)_img // UIImage *
                  forState:(UIControlState)_state {
}
- (id)thumbImageForState:(UIControlState)_state {
    nil
}
- (id)minimumTrackImageForState:(UIControlState)_state {
    nil
}
- (id)maximumTrackImageForState:(UIControlState)_state {
    nil
}
- (id)currentThumbImage {
    nil
}

- (f32)value {
    env.objc.borrow::<UISliderHostObject>(this).value
}
- (())setValue:(f32)value {
    env.objc.borrow_mut::<UISliderHostObject>(this).value = value;
}
- (())setValue:(f32)value animated:(bool)_animated {
    env.objc.borrow_mut::<UISliderHostObject>(this).value = value;
}
- (f32)minimumValue {
    env.objc.borrow::<UISliderHostObject>(this).minimum_value
}
- (())setMinimumValue:(f32)value {
    env.objc.borrow_mut::<UISliderHostObject>(this).minimum_value = value;
}
- (f32)maximumValue {
    env.objc.borrow::<UISliderHostObject>(this).maximum_value
}
- (())setMaximumValue:(f32)value {
    env.objc.borrow_mut::<UISliderHostObject>(this).maximum_value = value;
}

// Whether value-changed events are sent continuously while dragging, or only
// once the drag ends. The slider isn't rendered and so never generates drag
// events itself, but apps configure this while setting up the control and
// read it back, so the value has to round-trip.
- (bool)isContinuous {
    env.objc.borrow::<UISliderHostObject>(this).continuous
}
- (())setContinuous:(bool)continuous {
    env.objc.borrow_mut::<UISliderHostObject>(this).continuous = continuous;
}

// TODO: all of it

@end

};
