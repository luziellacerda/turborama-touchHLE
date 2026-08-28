/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */
//! `CGContext.h`

use super::cg_affine_transform::{CGAffineTransform, CGAffineTransformIdentity};
use super::cg_bitmap_context::CGBitmapContextDrawer;
use super::cg_color::CGColorRef;
use super::cg_color_space::{
    kCGColorSpaceModelMonochrome, kCGColorSpaceModelRGB, CGColorSpaceGetModel, CGColorSpaceRef,
};
use super::cg_font::{
    new_cg_font_from_kind, CGFontHostObject, CGFontRef, CGFontRelease, CGFontRetain, CGGlyph,
};
use super::cg_image::CGImageRef;
use super::{cg_bitmap_context, cg_color, CGFloat, CGPoint, CGRect, CGSize};
use crate::dyld::{export_c_func, FunctionExports};
use crate::frameworks::core_foundation::{CFRelease, CFRetain, CFTypeRef};
use crate::frameworks::foundation::unichar;
use crate::frameworks::uikit;
use crate::mem::{ConstPtr, GuestUSize};
use crate::objc::{objc_classes, ClassExports, HostObject};
use crate::Environment;

type CGInterpolationQuality = i32;

type CGTextDrawingMode = i32;
const kCGTextFill: CGTextDrawingMode = 0;
const kCGTextStroke: CGTextDrawingMode = 1;
const kCGTextFillStroke: CGTextDrawingMode = 2;
const kCGTextInvisible: CGTextDrawingMode = 3;
const kCGTextFillClip: CGTextDrawingMode = 4;
const kCGTextStrokeClip: CGTextDrawingMode = 5;
const kCGTextFillStrokeClip: CGTextDrawingMode = 6;
const kCGTextClip: CGTextDrawingMode = 7;

pub type CGBlendMode = i32;
pub const kCGBlendModeNormal: CGBlendMode = 0;
pub const kCGBlendModeMultiply: CGBlendMode = 1;
pub const kCGBlendModeScreen: CGBlendMode = 2;
#[allow(unused)]
pub const kCGBlendModeOverlay: CGBlendMode = 3;
pub const kCGBlendModeDarken: CGBlendMode = 4;
pub const kCGBlendModeLighten: CGBlendMode = 5;
pub const kCGBlendModeCopy: CGBlendMode = 17;

pub const CLASSES: ClassExports = objc_classes! {

(env, this, _cmd);

// CGContext seems to be a CFType-based type, but in our implementation those
// are just Objective-C types, so we need a class for it, but its name is not
// visible anywhere.
@implementation _touchHLE_CGContext: NSObject

- (())dealloc {
    let host_obj = env.objc.borrow::<CGContextHostObject>(this);
    let CGContextSubclass::CGBitmapContext(bitmap_data) = host_obj.subclass;
    if bitmap_data.data_is_owned {
        env.mem.free(bitmap_data.data);
    }
    CGFontRelease(env, host_obj.font);

    env.objc.dealloc_object(this, &mut env.mem)
}

@end

};

// TODO: keep more states saved once they are implemented
type ContextState = (
    (CGFloat, CGFloat, CGFloat, CGFloat), // RGB fill color
    CGAffineTransform,                    // transform
    CGFontRef,                            // font
    CGFloat,                              // font size
    CGBlendMode,                          // blend mode
    Option<CGRect>,                       // clip rect
);

pub(super) struct CGContextHostObject {
    pub(super) subclass: CGContextSubclass,
    pub(super) rgb_fill_color: (CGFloat, CGFloat, CGFloat, CGFloat),
    pub(super) rgb_stroke_color: (CGFloat, CGFloat, CGFloat, CGFloat),
    /// Not currently included in CGContextSaveGState/RestoreGState state,
    /// matching rgb_stroke_color above (which also isn't).
    pub(super) line_width: CGFloat,
    /// Current path under construction. Each element is a subpath: its
    /// points (in user space, as given to CGContextMoveToPoint /
    /// CGContextAddLineToPoint) plus whether CGContextClosePath was called
    /// on it. Cleared by CGContextBeginPath.
    pub(super) path: Vec<(Vec<CGPoint>, bool)>,
    /// The current clip region, as an axis-aligned rectangle in device
    /// space. `None` means unclipped (the whole bitmap). Only rectangular
    /// clips under an identity transform are supported (see
    /// CGContextClipToRect); anything else isn't representable here.
    /// Unlike line_width/rgb_stroke_color, this *is* included in
    /// CGContextSaveGState/RestoreGState, since clip-then-restore is a very
    /// common pairing and leaving a stale clip in place after a restore
    /// would be a worse bug than not saving line_width/rgb_stroke_color is.
    pub(super) clip_rect: Option<CGRect>,
    pub(super) font: CGFontRef,
    pub(super) font_size: CGFloat,
    /// Current transform.
    pub(super) transform: CGAffineTransform,
    pub(super) blend_mode: CGBlendMode,
    /// Text transform.
    pub(super) text_transform: Option<CGAffineTransform>,
    pub(super) state_stack: Vec<ContextState>,
}
impl HostObject for CGContextHostObject {}

pub(super) enum CGContextSubclass {
    CGBitmapContext(cg_bitmap_context::CGBitmapContextData),
}

pub type CGContextRef = CFTypeRef;

pub fn CGContextRelease(env: &mut Environment, c: CGContextRef) {
    if !c.is_null() {
        CFRelease(env, c);
    }
}
pub fn CGContextRetain(env: &mut Environment, c: CGContextRef) -> CGContextRef {
    if !c.is_null() {
        CFRetain(env, c)
    } else {
        c
    }
}

fn CGContextSetBlendMode(env: &mut Environment, context: CGContextRef, blend_mode: CGBlendMode) {
    env.objc
        .borrow_mut::<CGContextHostObject>(context)
        .blend_mode = blend_mode;
}

fn CGContextSetFillColorSpace(
    env: &mut Environment,
    _context: CGContextRef,
    space: CGColorSpaceRef,
) {
    let color_model = CGColorSpaceGetModel(env, space);
    assert!(color_model == kCGColorSpaceModelMonochrome || color_model == kCGColorSpaceModelRGB);
    // TODO
}

fn CGContextSetFillColorWithColor(env: &mut Environment, context: CGContextRef, color: CGColorRef) {
    let (r, g, b, a) = cg_color::to_rgba(&env.objc, color);
    CGContextSetRGBFillColor(env, context, r, g, b, a)
}

pub fn CGContextSetRGBFillColor(
    env: &mut Environment,
    context: CGContextRef,
    red: CGFloat,
    green: CGFloat,
    blue: CGFloat,
    alpha: CGFloat,
) {
    let color = (red, green, blue, alpha);
    env.objc
        .borrow_mut::<CGContextHostObject>(context)
        .rgb_fill_color = color;
}

fn CGContextSetGrayFillColor(
    env: &mut Environment,
    context: CGContextRef,
    gray: CGFloat,
    alpha: CGFloat,
) {
    let color = (gray, gray, gray, alpha);
    env.objc
        .borrow_mut::<CGContextHostObject>(context)
        .rgb_fill_color = color;
}

fn CGContextSetGrayStrokeColor(
    _env: &mut Environment,
    context: CGContextRef,
    gray: CGFloat,
    alpha: CGFloat,
) {
    log!(
        "TODO: CGContextSetGrayStrokeColor({:?}, {}, {})",
        context,
        gray,
        alpha,
    );
}
pub fn CGContextSetRGBStrokeColor(
    env: &mut Environment,
    context: CGContextRef,
    r: CGFloat,
    g: CGFloat,
    b: CGFloat,
    a: CGFloat,
) {
    let color = (r, g, b, a);
    env.objc
        .borrow_mut::<CGContextHostObject>(context)
        .rgb_stroke_color = color;
}

fn CGContextSetShadow(
    _env: &mut Environment,
    context: CGContextRef,
    offset: CGSize,
    blur: CGFloat,
) {
    // Shadows are purely decorative; skipping them costs some visual polish but
    // leaves the drawing itself correct.
    log!(
        "TODO: CGContextSetShadow({:?}, {:?}, {})",
        context,
        offset,
        blur
    );
}

fn CGContextSetShadowWithColor(
    _env: &mut Environment,
    context: CGContextRef,
    offset: CGSize,
    blur: CGFloat,
    color: CGColorRef,
) {
    log!(
        "TODO: CGContextSetShadowWithColor({:?}, {}, {}, {:?})",
        context,
        offset,
        blur,
        color
    );
}

pub fn CGContextFillRect(env: &mut Environment, context: CGContextRef, rect: CGRect) {
    cg_bitmap_context::fill_rect(env, context, rect, /* clear: */ false);
}

pub fn CGContextSetLineWidth(env: &mut Environment, context: CGContextRef, width: CGFloat) {
    env.objc
        .borrow_mut::<CGContextHostObject>(context)
        .line_width = width;
}

fn CGContextStrokeRect(env: &mut Environment, context: CGContextRef, rect: CGRect) {
    let line_width = env.objc.borrow::<CGContextHostObject>(context).line_width;
    CGContextStrokeRectWithWidth(env, context, rect, line_width);
}

pub fn CGContextStrokeRectWithWidth(
    env: &mut Environment,
    context: CGContextRef,
    rect: CGRect,
    line_width: CGFloat,
) {
    // touchHLE doesn't have real path-stroking infrastructure yet (no other
    // stroke operation is implemented either), so this approximates the
    // rectangle's outline by filling four thin rectangles, one per edge,
    // with the current stroke color. This matches the visible appearance of
    // a real CGContextStrokeRectWithWidth call for the common case of an
    // axis-aligned, unrotated context.
    let host_obj = env.objc.borrow::<CGContextHostObject>(context);
    let stroke_color = host_obj.rgb_stroke_color;
    let saved_fill_color = host_obj.rgb_fill_color;

    env.objc
        .borrow_mut::<CGContextHostObject>(context)
        .rgb_fill_color = stroke_color;

    let CGRect { origin, size } = rect;
    let w = line_width;
    let edges = [
        // Top edge
        CGRect {
            origin,
            size: CGSize {
                width: size.width,
                height: w,
            },
        },
        // Bottom edge
        CGRect {
            origin: CGPoint {
                x: origin.x,
                y: origin.y + size.height - w,
            },
            size: CGSize {
                width: size.width,
                height: w,
            },
        },
        // Left edge
        CGRect {
            origin,
            size: CGSize {
                width: w,
                height: size.height,
            },
        },
        // Right edge
        CGRect {
            origin: CGPoint {
                x: origin.x + size.width - w,
                y: origin.y,
            },
            size: CGSize {
                width: w,
                height: size.height,
            },
        },
    ];
    for edge in edges {
        cg_bitmap_context::fill_rect(env, context, edge, /* clear: */ false);
    }

    env.objc
        .borrow_mut::<CGContextHostObject>(context)
        .rgb_fill_color = saved_fill_color;
}

pub fn CGContextClearRect(env: &mut Environment, context: CGContextRef, rect: CGRect) {
    cg_bitmap_context::fill_rect(env, context, rect, /* clear: */ true);
}

pub(super) fn CGContextClipToRect(env: &mut Environment, context: CGContextRef, rect: CGRect) {
    let host_obj = env.objc.borrow::<CGContextHostObject>(context);

    // The clip is stored in device space (see CGContextHostObject::clip_rect),
    // so the rect has to be mapped through the current transform first.
    //
    // Under translation and scaling a rectangle stays a rectangle, so this is
    // exact. Rotation and skew turn it into a quadrilateral that cannot be
    // represented here; apply_to_rect() then yields the bounding box, which
    // clips away less than it should. That is a far better outcome than
    // aborting, and apps combining a rotated transform with a rect clip are
    // rare.
    let transform = host_obj.transform;
    let rect = if transform.is_identity() {
        rect
    } else {
        if transform.b != 0.0 || transform.c != 0.0 {
            log!(
                "TODO: CGContextClipToRect() under a rotated/skewed transform, \
                 clipping to the bounding box instead"
            );
        }
        transform.apply_to_rect(rect)
    };

    // CGContextClipToRect() intersects with whatever clip is already in
    // effect; it doesn't replace it.
    let new_clip_rect = match host_obj.clip_rect {
        Some(existing) => intersect_rects(existing, rect),
        None => rect,
    };
    env.objc
        .borrow_mut::<CGContextHostObject>(context)
        .clip_rect = Some(new_clip_rect);
}

fn intersect_rects(a: CGRect, b: CGRect) -> CGRect {
    let min_x = a.origin.x.max(b.origin.x);
    let min_y = a.origin.y.max(b.origin.y);
    let max_x = (a.origin.x + a.size.width).min(b.origin.x + b.size.width);
    let max_y = (a.origin.y + a.size.height).min(b.origin.y + b.size.height);
    CGRect {
        origin: CGPoint {
            x: min_x,
            y: min_y,
        },
        size: CGSize {
            // A disjoint intersection becomes a zero-size rect (clips out
            // everything) rather than a negative size.
            width: (max_x - min_x).max(0.0),
            height: (max_y - min_y).max(0.0),
        },
    }
}

pub fn CGContextConcatCTM(
    env: &mut Environment,
    context: CGContextRef,
    transform: CGAffineTransform,
) {
    log_dbg!("CGContextConcatCTM({:?})", transform);
    let host_obj = env.objc.borrow_mut::<CGContextHostObject>(context);
    host_obj.transform = transform.concat(host_obj.transform);
}
pub fn CGContextGetCTM(env: &mut Environment, context: CGContextRef) -> CGAffineTransform {
    let res = env.objc.borrow::<CGContextHostObject>(context).transform;
    log_dbg!("CGContextGetCTM() => {:?}", res);
    res
}
pub fn CGContextRotateCTM(env: &mut Environment, context: CGContextRef, angle: CGFloat) {
    log_dbg!("CGContextRotateCTM({:?})", angle);
    let host_obj = env.objc.borrow_mut::<CGContextHostObject>(context);
    host_obj.transform = host_obj.transform.rotate(angle);
}
pub fn CGContextScaleCTM(env: &mut Environment, context: CGContextRef, x: CGFloat, y: CGFloat) {
    log_dbg!("CGContextScaleCTM({:?})", (x, y));
    let host_obj = env.objc.borrow_mut::<CGContextHostObject>(context);
    host_obj.transform = host_obj.transform.scale(x, y);
}
pub fn CGContextTranslateCTM(
    env: &mut Environment,
    context: CGContextRef,
    tx: CGFloat,
    ty: CGFloat,
) {
    log_dbg!("CGContextTranslateCTM({:?})", (tx, ty));
    let host_obj = env.objc.borrow_mut::<CGContextHostObject>(context);
    host_obj.transform = host_obj.transform.translate(tx, ty);
}

pub fn CGContextDrawImage(
    env: &mut Environment,
    context: CGContextRef,
    rect: CGRect,
    image: CGImageRef,
) {
    cg_bitmap_context::draw_image(env, context, rect, image, /* alpha: */ 1.0);
}

/// Not a real CG function. Helper for UIImage's `drawAtPoint:blendMode:
/// alpha:` and `drawInRect:blendMode:alpha:`, which need a one-off blend
/// mode and alpha that aren't part of the plain `CGContextDrawImage` API
/// (real UIKit derives these from the current graphics context state,
/// which touchHLE doesn't fully model here). Temporarily overrides the
/// context's blend mode for the duration of the draw, then restores it.
pub fn CGContextDrawImageWithBlendModeAndAlpha(
    env: &mut Environment,
    context: CGContextRef,
    rect: CGRect,
    image: CGImageRef,
    blend_mode: CGBlendMode,
    alpha: CGFloat,
) {
    let old_blend_mode = env.objc.borrow::<CGContextHostObject>(context).blend_mode;
    env.objc.borrow_mut::<CGContextHostObject>(context).blend_mode = blend_mode;
    cg_bitmap_context::draw_image(env, context, rect, image, alpha);
    env.objc.borrow_mut::<CGContextHostObject>(context).blend_mode = old_blend_mode;
}

fn CGContextSaveGState(env: &mut Environment, context: CGContextRef) {
    let host_obj = env.objc.borrow_mut::<CGContextHostObject>(context);
    host_obj.state_stack.push((
        host_obj.rgb_fill_color,
        host_obj.transform,
        host_obj.font,
        host_obj.font_size,
        host_obj.blend_mode,
        host_obj.clip_rect,
    ));
    CGFontRetain(env, env.objc.borrow::<CGContextHostObject>(context).font);
}

fn CGContextRestoreGState(env: &mut Environment, context: CGContextRef) {
    // We need to release _old_ font, there are 2 cases:
    // - font hasn't been set between save/restore -> this release corresponds
    // the font retain from save
    // - font has been set between save/restore -> we need to release old font
    // retained on the set
    CGFontRelease(env, env.objc.borrow::<CGContextHostObject>(context).font);
    let host_obj = env.objc.borrow_mut::<CGContextHostObject>(context);
    let state = host_obj.state_stack.pop().unwrap();
    host_obj.rgb_fill_color = state.0;
    host_obj.transform = state.1;
    host_obj.font = state.2;
    host_obj.font_size = state.3;
    host_obj.blend_mode = state.4;
    host_obj.clip_rect = state.5;
}

fn CGContextSetInterpolationQuality(
    _env: &mut Environment,
    context: CGContextRef,
    quality: CGInterpolationQuality,
) {
    log!(
        "TODO: CGContextSetInterpolationQuality({:?}, {:?})",
        context,
        quality
    );
}
fn CGContextSetAllowsAntialiasing(_env: &mut Environment, context: CGContextRef, allow: bool) {
    log!(
        "TODO: CGContextSetAllowsAntialiasing({:?}, {})",
        context,
        allow
    );
}
fn CGContextSetShouldAntialias(_env: &mut Environment, context: CGContextRef, should: bool) {
    log!(
        "TODO: CGContextSetShouldAntialias({:?}, {})",
        context,
        should
    );
}

fn CGContextSetShouldSmoothFonts(_env: &mut Environment, context: CGContextRef, should: bool) {
    log!(
        "TODO: CGContextSetShouldSmoothFonts({:?}, {})",
        context,
        should
    );
}

fn CGContextSetFont(env: &mut Environment, context: CGContextRef, font: CGFontRef) {
    CGFontRetain(env, font);
    let old_font = env.objc.borrow_mut::<CGContextHostObject>(context).font;
    CGFontRelease(env, old_font);
    env.objc.borrow_mut::<CGContextHostObject>(context).font = font;
}

fn CGContextSetFontSize(env: &mut Environment, context: CGContextRef, size: CGFloat) {
    env.objc
        .borrow_mut::<CGContextHostObject>(context)
        .font_size = size;
}

fn CGContextSetTextDrawingMode(
    _env: &mut Environment,
    _context: CGContextRef,
    mode: CGTextDrawingMode,
) {
    // touchHLE always draws text filled. Stroking and the clipping modes are
    // not supported, but drawing filled text is much closer to what the app
    // wants than aborting, and kCGTextInvisible is only used for measuring, in
    // which case what gets drawn does not matter either.
    match mode {
        kCGTextFill | kCGTextFillStroke => {}
        kCGTextStroke | kCGTextInvisible | kCGTextFillClip | kCGTextStrokeClip
        | kCGTextFillStrokeClip | kCGTextClip => {
            log!(
                "TODO: CGContextSetTextDrawingMode() mode {}, drawing filled text instead",
                mode
            );
        }
        _ => {
            log!(
                "Warning: CGContextSetTextDrawingMode() unknown mode {}, ignoring",
                mode
            );
        }
    }
}

fn CGContextSetTextMatrix(
    env: &mut Environment,
    context: CGContextRef,
    transform: CGAffineTransform,
) {
    log_dbg!("CGContextSetTextMatrix({:?})", transform);
    env.objc
        .borrow_mut::<CGContextHostObject>(context)
        .text_transform = Some(transform);
}

fn CGContextGetTextPosition(env: &mut Environment, context: CGContextRef) -> CGPoint {
    // touchHLE doesn't track a running text position separately; derive it from
    // the text transform's translation, defaulting to the origin.
    let res = match env
        .objc
        .borrow::<CGContextHostObject>(context)
        .text_transform
    {
        Some(t) => CGPoint { x: t.tx, y: t.ty },
        None => CGPoint { x: 0.0, y: 0.0 },
    };
    log_dbg!("CGContextGetTextPosition() => {:?}", res);
    res
}

/// Legacy API combining `CGContextSetFont()` and `CGContextSetFontSize()`,
/// naming the font instead of taking a `CGFontRef`.
fn CGContextSelectFont(
    env: &mut Environment,
    context: CGContextRef,
    name: ConstPtr<u8>,
    size: CGFloat,
    text_encoding: i32,
) {
    let name_str = if name.is_null() {
        String::new()
    } else {
        env.mem.cstr_at_utf8(name).unwrap_or("").to_string()
    };

    // The encoding only matters for CGContextShowText(), which treats bytes as
    // MacRoman; for the ASCII range that apps actually use it makes no
    // difference, so it is noted and otherwise ignored.
    let kind = uikit::ui_font::get_equivalent_font(&name_str).unwrap_or_else(|| {
        log!(
            "CGContextSelectFont: no replacement for font {:?}, using the system font",
            name_str
        );
        uikit::ui_font::FontKind::SansRegular
    });
    log_dbg!(
        "CGContextSelectFont({:?}, {:?}, {}, encoding {})",
        context,
        name_str,
        size,
        text_encoding
    );

    let font = new_cg_font_from_kind(env, kind);
    CGContextSetFont(env, context, font);
    // CGContextSetFont() retains it, so drop our own reference.
    CGFontRelease(env, font);
    CGContextSetFontSize(env, context, size);
}

/// Legacy API drawing single-byte text at a point using the font previously
/// chosen with `CGContextSelectFont()`.
fn CGContextShowTextAtPoint(
    env: &mut Environment,
    context: CGContextRef,
    x: CGFloat,
    y: CGFloat,
    string: ConstPtr<u8>,
    length: GuestUSize,
) {
    let font = env.objc.borrow::<CGContextHostObject>(context).font;
    if font.is_null() {
        log!("CGContextShowTextAtPoint() with no font selected, ignoring");
        return;
    }

    // Bytes are interpreted as MacRoman, which agrees with Latin-1 and ASCII
    // over the range apps use in practice.
    let mut glyph_ids = Vec::with_capacity(length as usize);
    {
        let host_font = &env.objc.borrow::<CGFontHostObject>(font).font;
        for i in 0..length {
            let byte = env.mem.read(string + i);
            glyph_ids.push(host_font.glyph_id_for_char(byte as unichar));
        }
    }

    show_glyph_ids_at_point(env, context, x, y, glyph_ids);
}

fn CGContextShowText(
    env: &mut Environment,
    context: CGContextRef,
    string: ConstPtr<u8>,
    length: GuestUSize,
) {
    // Without text position tracking the only sensible origin is the current
    // one, which touchHLE keeps at zero; apps using this call generally set a
    // text matrix first.
    CGContextShowTextAtPoint(env, context, 0.0, 0.0, string, length);
}

fn CGContextShowGlyphsAtPoint(
    env: &mut Environment,
    context: CGContextRef,
    x: CGFloat,
    y: CGFloat,
    glyphs: ConstPtr<CGGlyph>,
    count: GuestUSize,
) {
    let mut glyph_ids = Vec::new();
    for i in 0..count {
        let glyph_id = env.mem.read(glyphs + i);
        glyph_ids.push(rusttype::GlyphId(glyph_id));
    }

    show_glyph_ids_at_point(env, context, x, y, glyph_ids);
}

/// Shared by the glyph-based and the legacy text-based drawing calls.
fn show_glyph_ids_at_point(
    env: &mut Environment,
    context: CGContextRef,
    x: CGFloat,
    y: CGFloat,
    glyph_ids: Vec<rusttype::GlyphId>,
) {
    let font = env.objc.borrow::<CGContextHostObject>(context).font;
    let font_size = env.objc.borrow::<CGContextHostObject>(context).font_size;
    let text_transform = env
        .objc
        .borrow::<CGContextHostObject>(context)
        .text_transform
        .unwrap_or(CGAffineTransformIdentity);

    let font = &env.objc.borrow::<CGFontHostObject>(font).font;

    let mut drawer = CGBitmapContextDrawer::new(&env.objc, &mut env.mem, context);
    let fill_color = drawer.rgb_fill_color();

    font.draw_glyphs(
        font_size,
        glyph_ids,
        (x, y),
        text_transform,
        |raster_glyph| {
            uikit::ui_font::draw_font_glyph(
                &mut drawer,
                raster_glyph,
                fill_color,
                /* clip_x: */ None,
                /* clip_y: */ None,
            )
        },
    );
}

fn CGContextShowGlyphsAtPositions(
    env: &mut Environment,
    context: CGContextRef,
    glyphs: ConstPtr<CGGlyph>,
    positions: ConstPtr<CGPoint>,
    count: GuestUSize,
) {
    let text_transform = env
        .objc
        .borrow::<CGContextHostObject>(context)
        .text_transform
        .unwrap_or(CGAffineTransformIdentity);
    assert!(text_transform.tx == 0.0 && text_transform.ty == 0.0); // TODO

    for i in 0..count {
        let glyph_ptr = glyphs + i;
        let point = env.mem.read(positions + i);
        let transformed_point = text_transform.apply_to_point(point);
        CGContextShowGlyphsAtPoint(
            env,
            context,
            transformed_point.x,
            transformed_point.y,
            glyph_ptr,
            1,
        );
    }
}

// CGContextFlush forces any pending drawing to be rendered. For a bitmap
// context (the only kind touchHLE has), drawing operations write straight into
// the pixel buffer in memory, so there is no deferred pipeline to flush and
// this is a no-op.
fn CGContextFlush(_env: &mut Environment, _context: CGContextRef) {
    // no-op
}

pub const FUNCTIONS: FunctionExports = &[
    export_c_func!(CGContextRetain(_)),
    export_c_func!(CGContextRelease(_)),
    export_c_func!(CGContextFlush(_)),
    export_c_func!(CGContextSetBlendMode(_, _)),
    export_c_func!(CGContextSetFillColorSpace(_, _)),
    export_c_func!(CGContextSetFillColorWithColor(_, _)),
    export_c_func!(CGContextSetRGBFillColor(_, _, _, _, _)),
    export_c_func!(CGContextSetGrayFillColor(_, _, _)),
    export_c_func!(CGContextSetGrayStrokeColor(_, _, _)),
    export_c_func!(CGContextSetRGBStrokeColor(_, _, _, _, _)),
    export_c_func!(CGContextSetLineWidth(_, _)),
    export_c_func!(CGContextSetShadowWithColor(_, _, _, _)),
    export_c_func!(CGContextFillRect(_, _)),
    export_c_func!(CGContextStrokeRect(_, _)),
    export_c_func!(CGContextStrokeRectWithWidth(_, _, _)),
    export_c_func!(CGContextClearRect(_, _)),
    export_c_func!(CGContextClipToRect(_, _)),
    export_c_func!(CGContextConcatCTM(_, _)),
    export_c_func!(CGContextGetCTM(_)),
    export_c_func!(CGContextRotateCTM(_, _)),
    export_c_func!(CGContextScaleCTM(_, _, _)),
    export_c_func!(CGContextTranslateCTM(_, _, _)),
    export_c_func!(CGContextDrawImage(_, _, _)),
    export_c_func!(CGContextSaveGState(_)),
    export_c_func!(CGContextRestoreGState(_)),
    export_c_func!(CGContextSetInterpolationQuality(_, _)),
    export_c_func!(CGContextSetAllowsAntialiasing(_, _)),
    export_c_func!(CGContextSetShouldAntialias(_, _)),
    export_c_func!(CGContextSetShouldSmoothFonts(_, _)),
    export_c_func!(CGContextSetShadow(_, _, _)),
    export_c_func!(CGContextSetFont(_, _)),
    export_c_func!(CGContextSetFontSize(_, _)),
    export_c_func!(CGContextSetTextDrawingMode(_, _)),
    export_c_func!(CGContextSetTextMatrix(_, _)),
    export_c_func!(CGContextGetTextPosition(_)),
    export_c_func!(CGContextSelectFont(_, _, _, _)),
    export_c_func!(CGContextShowTextAtPoint(_, _, _, _, _)),
    export_c_func!(CGContextShowText(_, _, _)),
    export_c_func!(CGContextShowGlyphsAtPoint(_, _, _, _, _)),
    export_c_func!(CGContextShowGlyphsAtPositions(_, _, _, _)),
];
