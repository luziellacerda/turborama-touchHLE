/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */
//! `UIGraphics.h`

use crate::dyld::{export_c_func, FunctionExports};
use crate::frameworks::core_graphics::cg_bitmap_context::{
    CGBitmapContextCreate, CGBitmapContextCreateImage,
};
use crate::frameworks::core_graphics::cg_color_space::{
    CGColorSpaceCreateDeviceRGB, CGColorSpaceRelease,
};
use crate::frameworks::core_graphics::cg_context::{
    CGContextRef, CGContextRelease, CGContextRetain,
};
use crate::frameworks::core_graphics::cg_image::{kCGImageAlphaPremultipliedLast, CGImageRelease};
use crate::frameworks::core_graphics::{CGFloat, CGSize};
use crate::mem::Ptr;
use crate::objc::{id, msg_class, nil};
use crate::Environment;

#[derive(Default)]
pub(super) struct State {
    pub(super) context_stack: Vec<CGContextRef>,
}

pub fn UIGraphicsPushContext(env: &mut Environment, context: CGContextRef) {
    CGContextRetain(env, context);
    env.framework_state
        .uikit
        .ui_graphics
        .context_stack
        .push(context);
}
pub fn UIGraphicsPopContext(env: &mut Environment) {
    let context = env.framework_state.uikit.ui_graphics.context_stack.pop();
    CGContextRelease(env, context.unwrap());
}
pub fn UIGraphicsGetCurrentContext(env: &mut Environment) -> CGContextRef {
    env.framework_state
        .uikit
        .ui_graphics
        .context_stack
        .last()
        .copied()
        .unwrap_or(nil)
}

fn UIGraphicsBeginImageContext(env: &mut Environment, size: CGSize) {
    UIGraphicsBeginImageContextWithOptions(env, size, false, 0.0);
}

fn UIGraphicsBeginImageContextWithOptions(
    env: &mut Environment,
    size: CGSize,
    _opaque: bool, // TODO: skip clearing to transparent black if true
    scale: CGFloat,
) {
    // A scale of 0.0 means "use the device's main screen scale". touchHLE
    // doesn't otherwise model @2x/@3x screens (see UIImage's `scale`
    // accessor, which is always 1.0), so 1.0 is used unconditionally here
    // too.
    let scale = if scale == 0.0 { 1.0 } else { scale };
    let width = (size.width * scale).round() as u32;
    let height = (size.height * scale).round() as u32;

    let color_space = CGColorSpaceCreateDeviceRGB(env);
    let context = CGBitmapContextCreate(
        env,
        Ptr::null(),
        width,
        height,
        8,
        0,
        color_space,
        kCGImageAlphaPremultipliedLast,
    );
    CGColorSpaceRelease(env, color_space);

    UIGraphicsPushContext(env, context);
    // UIGraphicsPushContext() retains the context, so this drops the extra
    // ref from CGBitmapContextCreate(), leaving the stack as sole owner.
    CGContextRelease(env, context);
}

fn UIGraphicsEndImageContext(env: &mut Environment) {
    UIGraphicsPopContext(env);
}

fn UIGraphicsGetImageFromCurrentImageContext(env: &mut Environment) -> id {
    let context = UIGraphicsGetCurrentContext(env);
    if context == nil {
        return nil;
    }
    let cg_image = CGBitmapContextCreateImage(env, context);
    if cg_image.is_null() {
        return nil;
    }
    let image: id = msg_class![env; UIImage imageWithCGImage:cg_image];
    CGImageRelease(env, cg_image);
    image
}

pub const FUNCTIONS: FunctionExports = &[
    export_c_func!(UIGraphicsPushContext(_)),
    export_c_func!(UIGraphicsPopContext()),
    export_c_func!(UIGraphicsGetCurrentContext()),
    export_c_func!(UIGraphicsBeginImageContext(_)),
    export_c_func!(UIGraphicsBeginImageContextWithOptions(_, _, _)),
    export_c_func!(UIGraphicsEndImageContext()),
    export_c_func!(UIGraphicsGetImageFromCurrentImageContext()),
];
