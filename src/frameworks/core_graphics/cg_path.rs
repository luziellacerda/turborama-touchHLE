/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */
//! Path construction and stroking, part of `CGContext.h`.
//!
//! touchHLE doesn't have a real vector path renderer (no bezier curves, no
//! polygon fill, no anti-aliasing). What's implemented here only covers
//! straight-line subpaths and stroking them, which is enough for the
//! common case of apps drawing simple borders, separators or line-art
//! icons. Curve-adding functions (CGContextAddCurveToPoint etc.) and
//! CGContextFillPath aren't implemented yet; they can be added the same
//! way once something actually needs them.

use super::cg_affine_transform::CGAffineTransform;
use super::cg_bitmap_context::CGBitmapContextDrawer;
use super::cg_context::{CGContextClipToRect, CGContextHostObject, CGContextRef};
use super::{CGFloat, CGPoint, CGRect, CGSize};
use crate::dyld::{export_c_func, FunctionExports};
use crate::frameworks::core_foundation::{CFRelease, CFRetain, CFTypeRef};
use crate::image::gamma_decode;
use crate::objc::{objc_classes, ClassExports, HostObject};
use crate::Environment;

fn CGContextBeginPath(env: &mut Environment, context: CGContextRef) {
    env.objc
        .borrow_mut::<CGContextHostObject>(context)
        .path
        .clear();
}

fn CGContextMoveToPoint(env: &mut Environment, context: CGContextRef, x: CGFloat, y: CGFloat) {
    env.objc
        .borrow_mut::<CGContextHostObject>(context)
        .path
        .push((vec![CGPoint { x, y }], false));
}

fn CGContextAddLineToPoint(env: &mut Environment, context: CGContextRef, x: CGFloat, y: CGFloat) {
    let host_obj = env.objc.borrow_mut::<CGContextHostObject>(context);
    match host_obj.path.last_mut() {
        Some((points, _closed)) => points.push(CGPoint { x, y }),
        // The app didn't call CGContextMoveToPoint() first. Real CG's
        // behaviour here isn't useful to emulate exactly; starting a new
        // subpath is a harmless fallback that avoids losing the point.
        None => host_obj.path.push((vec![CGPoint { x, y }], false)),
    }
}

fn CGContextClosePath(env: &mut Environment, context: CGContextRef) {
    if let Some((_points, closed)) = env
        .objc
        .borrow_mut::<CGContextHostObject>(context)
        .path
        .last_mut()
    {
        *closed = true;
    }
}

// touchHLE has no real per-shape clip mask (see CGContextHostObject::
// clip_rect); what's implemented here is an approximation: the current
// path's bounding box is used as the clip rectangle instead of its exact
// shape. For a path that's already an axis-aligned rectangle (a very
// common case: CGContextAddRect() then CGContextClip()) this is exact; for
// anything else (rounded rects, arbitrary shapes) it clips to the
// rectangular bounds rather than the precise outline, which under-clips
// (shows some pixels that should have been clipped away) rather than
// crashing or drawing nothing.
fn CGContextClip(env: &mut Environment, context: CGContextRef) {
    let subpaths = env.objc.borrow::<CGContextHostObject>(context).path.clone();

    // CGContextClip() empties the current path afterward, regardless of
    // whether the clip region could actually be represented.
    env.objc
        .borrow_mut::<CGContextHostObject>(context)
        .path
        .clear();

    let mut min_x = f32::INFINITY;
    let mut min_y = f32::INFINITY;
    let mut max_x = f32::NEG_INFINITY;
    let mut max_y = f32::NEG_INFINITY;
    for (points, _closed) in &subpaths {
        for p in points {
            min_x = min_x.min(p.x);
            min_y = min_y.min(p.y);
            max_x = max_x.max(p.x);
            max_y = max_y.max(p.y);
        }
    }
    if !min_x.is_finite() {
        return; // Empty path: nothing to clip.
    }

    let rect = CGRect {
        origin: CGPoint {
            x: min_x,
            y: min_y,
        },
        size: CGSize {
            width: max_x - min_x,
            height: max_y - min_y,
        },
    };
    CGContextClipToRect(env, context, rect);
}

fn CGContextStrokePath(env: &mut Environment, context: CGContextRef) {
    let &CGContextHostObject {
        transform,
        line_width,
        rgb_stroke_color,
        ..
    } = env.objc.borrow::<CGContextHostObject>(context);
    let subpaths = env.objc.borrow::<CGContextHostObject>(context).path.clone();

    let color = (
        gamma_decode(rgb_stroke_color.0),
        gamma_decode(rgb_stroke_color.1),
        gamma_decode(rgb_stroke_color.2),
        rgb_stroke_color.3,
    );

    let mut drawer = CGBitmapContextDrawer::new(&env.objc, &mut env.mem, context);

    for (points, closed) in &subpaths {
        if points.len() < 2 {
            continue;
        }
        for window in points.windows(2) {
            stroke_segment(&mut drawer, transform, window[0], window[1], line_width, color);
        }
        if *closed {
            stroke_segment(
                &mut drawer,
                transform,
                *points.last().unwrap(),
                points[0],
                line_width,
                color,
            );
        }
    }
}

/// Draws a single straight, thick line segment by filling the rectangular
/// quad it covers. This is the same "approximate a stroke by filling a
/// shape with the fill-rect machinery" idea already used by
/// CGContextStrokeRectWithWidth, just generalised from axis-aligned
/// rectangle edges to an arbitrary segment.
fn stroke_segment(
    drawer: &mut CGBitmapContextDrawer,
    transform: CGAffineTransform,
    a: CGPoint,
    b: CGPoint,
    width: CGFloat,
    color: (CGFloat, CGFloat, CGFloat, CGFloat),
) {
    let a = transform.apply_to_point(a);
    let b = transform.apply_to_point(b);

    let (dx, dy) = (b.x - a.x, b.y - a.y);
    let len = (dx * dx + dy * dy).sqrt();
    if len == 0.0 {
        return;
    }
    let (nx, ny) = (-dy / len, dx / len); // unit vector perpendicular to a->b
    let half_width = width.max(1.0) / 2.0;
    let corners = [
        CGPoint {
            x: a.x + nx * half_width,
            y: a.y + ny * half_width,
        },
        CGPoint {
            x: b.x + nx * half_width,
            y: b.y + ny * half_width,
        },
        CGPoint {
            x: b.x - nx * half_width,
            y: b.y - ny * half_width,
        },
        CGPoint {
            x: a.x - nx * half_width,
            y: a.y - ny * half_width,
        },
    ];
    fill_convex_quad(drawer, &corners, color);
}

fn fill_convex_quad(
    drawer: &mut CGBitmapContextDrawer,
    corners: &[CGPoint; 4],
    color: (CGFloat, CGFloat, CGFloat, CGFloat),
) {
    let min_x = corners
        .iter()
        .map(|p| p.x)
        .fold(f32::INFINITY, f32::min)
        .floor()
        .max(0.0) as i32;
    let min_y = corners
        .iter()
        .map(|p| p.y)
        .fold(f32::INFINITY, f32::min)
        .floor()
        .max(0.0) as i32;
    let max_x = (corners
        .iter()
        .map(|p| p.x)
        .fold(f32::NEG_INFINITY, f32::max)
        .ceil() as i32)
        .min(drawer.width() as i32 - 1);
    let max_y = (corners
        .iter()
        .map(|p| p.y)
        .fold(f32::NEG_INFINITY, f32::max)
        .ceil() as i32)
        .min(drawer.height() as i32 - 1);

    for y in min_y..=max_y {
        for x in min_x..=max_x {
            let p = CGPoint {
                x: x as f32 + 0.5,
                y: y as f32 + 0.5,
            };
            if point_in_convex_quad(p, corners) {
                drawer.put_pixel((x, y), color, /* blend: */ true);
            }
        }
    }
}

/// Same-sign-of-cross-product test: works for both clockwise and
/// counter-clockwise convex quads.
fn point_in_convex_quad(p: CGPoint, corners: &[CGPoint; 4]) -> bool {
    let mut sign = 0.0f32;
    for i in 0..4 {
        let a = corners[i];
        let b = corners[(i + 1) % 4];
        let cross = (b.x - a.x) * (p.y - a.y) - (b.y - a.y) * (p.x - a.x);
        if cross != 0.0 {
            if sign == 0.0 {
                sign = cross.signum();
            } else if cross.signum() != sign {
                return false;
            }
        }
    }
    true
}

pub const CLASSES: ClassExports = objc_classes! {

(env, this, _cmd);

// CGPath is a CFType-based type; like CGColor we back it with an Obj-C class
// whose name isn't visible anywhere.
@implementation _touchHLE_CGPath: NSObject
@end

};

/// Subpaths in the same representation the context uses: a list of
/// (points, closed) tuples.
pub struct CGPathHostObject {
    pub subpaths: Vec<(Vec<CGPoint>, bool)>,
}
impl HostObject for CGPathHostObject {}

pub type CGPathRef = CFTypeRef;
pub type CGMutablePathRef = CFTypeRef;

fn new_path(env: &mut Environment, subpaths: Vec<(Vec<CGPoint>, bool)>) -> CGMutablePathRef {
    let host_obj = Box::new(CGPathHostObject { subpaths });
    let class = env.objc.get_known_class("_touchHLE_CGPath", &mut env.mem);
    env.objc.alloc_object(class, host_obj, &mut env.mem)
}

fn CGPathCreateMutable(env: &mut Environment) -> CGMutablePathRef {
    new_path(env, Vec::new())
}

fn CGPathCreateMutableCopy(env: &mut Environment, path: CGPathRef) -> CGMutablePathRef {
    let subpaths = if path.is_null() {
        Vec::new()
    } else {
        env.objc.borrow::<CGPathHostObject>(path).subpaths.clone()
    };
    new_path(env, subpaths)
}

fn CGPathRelease(env: &mut Environment, path: CGPathRef) {
    if !path.is_null() {
        CFRelease(env, path);
    }
}
fn CGPathRetain(env: &mut Environment, path: CGPathRef) -> CGPathRef {
    if !path.is_null() {
        CFRetain(env, path)
    } else {
        path
    }
}

fn CGPathMoveToPoint(
    env: &mut Environment,
    path: CGMutablePathRef,
    _m: crate::mem::ConstPtr<CGAffineTransform>,
    x: CGFloat,
    y: CGFloat,
) {
    // The optional transform is ignored (touchHLE's path model is minimal).
    env.objc
        .borrow_mut::<CGPathHostObject>(path)
        .subpaths
        .push((vec![CGPoint { x, y }], false));
}

fn CGPathAddLineToPoint(
    env: &mut Environment,
    path: CGMutablePathRef,
    _m: crate::mem::ConstPtr<CGAffineTransform>,
    x: CGFloat,
    y: CGFloat,
) {
    let host_obj = env.objc.borrow_mut::<CGPathHostObject>(path);
    match host_obj.subpaths.last_mut() {
        Some((points, _closed)) => points.push(CGPoint { x, y }),
        None => host_obj.subpaths.push((vec![CGPoint { x, y }], false)),
    }
}

fn CGPathCloseSubpath(env: &mut Environment, path: CGMutablePathRef) {
    if let Some((_points, closed)) = env
        .objc
        .borrow_mut::<CGPathHostObject>(path)
        .subpaths
        .last_mut()
    {
        *closed = true;
    }
}

fn CGPathAddRect(
    env: &mut Environment,
    path: CGMutablePathRef,
    _m: crate::mem::ConstPtr<CGAffineTransform>,
    rect: super::CGRect,
) {
    let x0 = rect.origin.x;
    let y0 = rect.origin.y;
    let x1 = x0 + rect.size.width;
    let y1 = y0 + rect.size.height;
    env.objc.borrow_mut::<CGPathHostObject>(path).subpaths.push((
        vec![
            CGPoint { x: x0, y: y0 },
            CGPoint { x: x1, y: y0 },
            CGPoint { x: x1, y: y1 },
            CGPoint { x: x0, y: y1 },
        ],
        true,
    ));
}

fn CGContextAddPath(env: &mut Environment, context: CGContextRef, path: CGPathRef) {
    if path.is_null() {
        return;
    }
    let subpaths = env.objc.borrow::<CGPathHostObject>(path).subpaths.clone();
    env.objc
        .borrow_mut::<CGContextHostObject>(context)
        .path
        .extend(subpaths);
}

pub const FUNCTIONS: FunctionExports = &[
    export_c_func!(CGContextBeginPath(_)),
    export_c_func!(CGContextMoveToPoint(_, _, _)),
    export_c_func!(CGContextAddLineToPoint(_, _, _)),
    export_c_func!(CGContextClosePath(_)),
    export_c_func!(CGContextClip(_)),
    export_c_func!(CGContextStrokePath(_)),
    export_c_func!(CGPathCreateMutable()),
    export_c_func!(CGPathCreateMutableCopy(_)),
    export_c_func!(CGPathRetain(_)),
    export_c_func!(CGPathRelease(_)),
    export_c_func!(CGPathMoveToPoint(_, _, _, _)),
    export_c_func!(CGPathAddLineToPoint(_, _, _, _)),
    export_c_func!(CGPathCloseSubpath(_)),
    export_c_func!(CGPathAddRect(_, _, _)),
    export_c_func!(CGContextAddPath(_, _)),
];
