/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */
//! `CGGradient.h`

use super::cg_bitmap_context::CGBitmapContextDrawer;
use super::cg_color;
use super::cg_color_space::{kCGColorSpaceGenericRGB, CGColorSpaceHostObject, CGColorSpaceRef};
use super::cg_context::{CGContextHostObject, CGContextRef};
use super::{CGFloat, CGPoint};
use crate::dyld::{export_c_func, FunctionExports};
use crate::frameworks::core_foundation::{CFRelease, CFRetain, CFTypeRef};
use crate::frameworks::foundation::NSUInteger;
use crate::image::gamma_decode;
use crate::mem::ConstPtr;
use crate::objc::{id, msg, objc_classes, ClassExports, HostObject};
use crate::Environment;

pub const CLASSES: ClassExports = objc_classes! {

(env, this, _cmd);

// Like CGColor, CGGradient is a CFType-based type without a visible class
// name in the real framework, so a fake class is used to host it here.
@implementation _touchHLE_CGGradient: NSObject
@end

};

/// Colour stop, already gamma-decoded to linear RGB. Alpha is always linear.
struct GradientStop {
    location: CGFloat,
    color: (CGFloat, CGFloat, CGFloat, CGFloat),
}

pub struct CGGradientHostObject {
    /// Sorted ascending by `location`. Never empty.
    stops: Vec<GradientStop>,
}
impl HostObject for CGGradientHostObject {}

pub type CGGradientRef = CFTypeRef;

pub fn CGGradientRetain(env: &mut Environment, gradient: CGGradientRef) -> CGGradientRef {
    if !gradient.is_null() {
        CFRetain(env, gradient)
    } else {
        gradient
    }
}
pub fn CGGradientRelease(env: &mut Environment, gradient: CGGradientRef) {
    if !gradient.is_null() {
        CFRelease(env, gradient);
    }
}

fn CGGradientCreateWithColors(
    env: &mut Environment,
    space: CGColorSpaceRef,
    colors: id,                   // CFArrayRef of CGColorRef, toll-free bridged to NSArray
    locations: ConstPtr<CGFloat>, // may be null
) -> CGGradientRef {
    if !space.is_null() {
        let name = env.objc.borrow::<CGColorSpaceHostObject>(space).name;
        assert_eq!(name, kCGColorSpaceGenericRGB); // TODO: other colour spaces
    }

    let count: NSUInteger = msg![env; colors count];
    assert!(count >= 1);

    let mut stops = Vec::with_capacity(count as usize);
    for i in 0..count {
        let color: id = msg![env; colors objectAtIndex:i];
        let (r, g, b, a) = cg_color::to_rgba(&env.objc, color);
        let location = if locations.is_null() {
            // Evenly spaced across [0, 1], matching the documented behaviour
            // of CGGradientCreateWithColors() when `locations` is NULL.
            if count == 1 {
                0.0
            } else {
                i as CGFloat / (count - 1) as CGFloat
            }
        } else {
            env.mem.read(locations + i)
        };
        stops.push(GradientStop {
            location,
            color: (gamma_decode(r), gamma_decode(g), gamma_decode(b), a),
        });
    }
    stops.sort_by(|a, b| a.location.partial_cmp(&b.location).unwrap());

    let host_object = Box::new(CGGradientHostObject { stops });
    let class = env
        .objc
        .get_known_class("_touchHLE_CGGradient", &mut env.mem);
    env.objc.alloc_object(class, host_object, &mut env.mem)
}

/// Sample the gradient at `t`. Values outside `[0, 1]` extrapolate using the
/// nearest stop's colour (callers wanting CG's draws-before/after-location
/// behaviour should clamp or skip before calling this). Returns linear RGB
/// with linear alpha.
fn sample(stops: &[GradientStop], t: CGFloat) -> (CGFloat, CGFloat, CGFloat, CGFloat) {
    let last = stops.len() - 1;
    if t <= stops[0].location {
        return stops[0].color;
    }
    if t >= stops[last].location {
        return stops[last].color;
    }
    for window in stops.windows(2) {
        let (a, b) = (&window[0], &window[1]);
        if t >= a.location && t <= b.location {
            let range = b.location - a.location;
            let frac = if range > 0.0 {
                (t - a.location) / range
            } else {
                0.0
            };
            return (
                a.color.0 + (b.color.0 - a.color.0) * frac,
                a.color.1 + (b.color.1 - a.color.1) * frac,
                a.color.2 + (b.color.2 - a.color.2) * frac,
                a.color.3 + (b.color.3 - a.color.3) * frac,
            );
        }
    }
    stops[last].color
}

pub type CGGradientDrawingOptions = u32;
pub const kCGGradientDrawsBeforeStartLocation: CGGradientDrawingOptions = 1 << 0;
pub const kCGGradientDrawsAfterEndLocation: CGGradientDrawingOptions = 1 << 1;

fn CGContextDrawLinearGradient(
    env: &mut Environment,
    context: CGContextRef,
    gradient: CGGradientRef,
    start_point: CGPoint,
    end_point: CGPoint,
    options: CGGradientDrawingOptions,
) {
    let draws_before = options & kCGGradientDrawsBeforeStartLocation != 0;
    let draws_after = options & kCGGradientDrawsAfterEndLocation != 0;

    // Gradient points are in user space, so the context's current transform
    // applies to them, same as it would to any other drawing operation.
    let &CGContextHostObject { transform, .. } = env.objc.borrow::<CGContextHostObject>(context);
    let start = transform.apply_to_point(start_point);
    let end = transform.apply_to_point(end_point);

    let stops = &env.objc.borrow::<CGGradientHostObject>(gradient).stops;
    // Cloned out so the borrow of `env.objc` doesn't overlap with the
    // drawer's borrow below.
    let stops: Vec<GradientStop> = stops
        .iter()
        .map(|s| GradientStop {
            location: s.location,
            color: s.color,
        })
        .collect();

    let mut drawer = CGBitmapContextDrawer::new(&env.objc, &mut env.mem, context);

    let (dx, dy) = (end.x - start.x, end.y - start.y);
    let len_sq = dx * dx + dy * dy;

    // touchHLE doesn't implement clip paths beyond the trivial "clip to the
    // whole bitmap" case (see CGContextClipToRect), so this always fills the
    // entire bitmap, matching what an unclipped gradient draw would do.
    let (width, height) = (drawer.width() as i32, drawer.height() as i32);
    for y in 0..height {
        for x in 0..width {
            let (px, py) = (x as f32 + 0.5, y as f32 + 0.5);
            let t = if len_sq > 0.0 {
                ((px - start.x) * dx + (py - start.y) * dy) / len_sq
            } else {
                0.0
            };
            if t < 0.0 && !draws_before {
                continue;
            }
            if t > 1.0 && !draws_after {
                continue;
            }
            let color = sample(&stops, t.clamp(0.0, 1.0));
            drawer.put_pixel((x, y), color, /* blend: */ true);
        }
    }
}

pub const FUNCTIONS: FunctionExports = &[
    export_c_func!(CGGradientRetain(_)),
    export_c_func!(CGGradientRelease(_)),
    export_c_func!(CGGradientCreateWithColors(_, _, _)),
    export_c_func!(CGContextDrawLinearGradient(_, _, _, _, _)),
];
