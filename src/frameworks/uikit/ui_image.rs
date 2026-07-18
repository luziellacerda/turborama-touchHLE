/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */
//! `UIImage`.

use crate::frameworks::core_graphics::cg_context::{
    CGBlendMode, CGContextDrawImage, CGContextDrawImageWithBlendModeAndAlpha,
};
use crate::frameworks::core_graphics::cg_image::{
    self, CGImageGetHeight, CGImageGetWidth, CGImageRef, CGImageRelease,
    CGImageRetain,
};
use crate::frameworks::core_graphics::{CGFloat, CGPoint, CGRect, CGSize};
use crate::frameworks::foundation::ns_string::get_static_str;
use crate::frameworks::foundation::{ns_data, ns_string, NSInteger, NSUInteger};
use crate::frameworks::uikit::ui_graphics::UIGraphicsGetCurrentContext;
use crate::dyld::{export_c_func, FunctionExports};
use crate::fs::GuestPath;
use crate::image::Image;
use crate::mem::MutVoidPtr;
use crate::objc::{
    autorelease, id, msg, msg_class, nil, objc_classes, release, retain, ClassExports, HostObject,
    NSZonePtr,
};
use crate::Environment;
use std::collections::HashMap;

const CACHE_SIZE: usize = 60;

#[derive(Default)]
pub struct State {
    /// Cache of images for `[UIImage imageNamed:]` method.
    /// Images are explicitly retained.
    cached_images: HashMap<String, id>,
}
impl State {
    fn get(env: &Environment) -> &Self {
        &env.framework_state.uikit.ui_image
    }
    fn get_mut(env: &mut Environment) -> &mut Self {
        &mut env.framework_state.uikit.ui_image
    }
}

struct UIImageHostObject {
    cg_image: CGImageRef,
}
impl HostObject for UIImageHostObject {}

pub const CLASSES: ClassExports = objc_classes! {

(env, this, _cmd);

@implementation UIImage: NSObject

+ (id)allocWithZone:(NSZonePtr)_zone {
    let host_object = Box::new(UIImageHostObject { cg_image: nil });
    env.objc.alloc_object(this, host_object, &mut env.mem)
}

+ (id)imageWithCGImage:(CGImageRef)cg_image {
    let new: id = msg![env; this alloc];
    let new: id = msg![env; new initWithCGImage:cg_image];
    autorelease(env, new)
}

+ (id)imageNamed:(id)name { // NSString*
    // TODO: figure out whether this is actually correct in all cases
    let bundle: id = msg_class![env; NSBundle mainBundle];
    let path: id = msg![env; bundle pathForResource:name ofType:nil];
    let name_str = ns_string::to_rust_string(env, name).to_string();
    if path == nil {
        log!("Warning: [UIImage imageNamed:{:?}] => nil", name_str);
        return nil;
    }
    // TODO: find a better eviction policy
    if State::get(env).cached_images.len() > CACHE_SIZE {
        let cache = std::mem::take(&mut State::get_mut(env).cached_images);
        log_dbg!("Evicting {} images from UIImage cache.", cache.len());
        for (_, img) in cache {
            release(env, img);
        }
    }
    if !State::get(env).cached_images.contains_key(&name_str) {
        let img = msg![env; this imageWithContentsOfFile:path];
        retain(env, img);
        State::get_mut(env).cached_images.insert(name_str.clone(), img);
    }
    *State::get(env).cached_images.get(&name_str).unwrap()
}

+ (id)imageWithContentsOfFile:(id)path { // NSString*
    let new: id = msg![env; this alloc];
    let new: id = msg![env; new initWithContentsOfFile:path];
    autorelease(env, new)
}

+ (id)imageWithData:(id)data { // NSData*
    let new: id = msg![env; this alloc];
    let new: id = msg![env; new initWithData:data];
    autorelease(env, new)
}

- (())dealloc {
    let &UIImageHostObject { cg_image } = env.objc.borrow(this);
    CGImageRelease(env, cg_image);

    env.objc.dealloc_object(this, &mut env.mem)
}

- (id)initWithCGImage:(CGImageRef)cg_image {
    CGImageRetain(env, cg_image);
    env.objc.borrow_mut::<UIImageHostObject>(this).cg_image = cg_image;
    this
}

- (id)initWithContentsOfFile:(id)path { // NSString*
    if path == nil {
        return nil;
    }
    let path = ns_string::to_rust_string(env, path); // TODO: avoid copy
    let Ok(bytes) = env.fs.read(GuestPath::new(&path)) else {
        log!("Warning: couldn't read image file at {:?}, returning nil", path);
        release(env, this);
        return nil;
    };
    // TODO: Real error handling. For now, most errors are likely to be caused
    //       by a functionality gap in touchHLE, not the app actually trying to
    //       load a broken file, so panicking is most useful.
    let image = Image::from_bytes(&bytes).unwrap();
    let cg_image = cg_image::from_image(env, image);
    env.objc.borrow_mut::<UIImageHostObject>(this).cg_image = cg_image;
    this
}

- (id)initWithData:(id)data { // NSData*
    let slice = ns_data::to_rust_slice(env, data);
    // TODO: refactor common parts
    let image = Image::from_bytes(slice).unwrap();
    let cg_image = cg_image::from_image(env, image);
    env.objc.borrow_mut::<UIImageHostObject>(this).cg_image = cg_image;
    this
}

- (id)stretchableImageWithLeftCapWidth:(NSInteger)_leftCapWidth
                          topCapHeight:(NSInteger)_topCapHeight {
    log!("TODO: properly support stretchableImageWithLeftCapWidth:topCapHeight:");
    retain(env, this)
}

// TODO: more init methods
// TODO: more accessors

- (CGImageRef)CGImage {
    env.objc.borrow::<UIImageHostObject>(this).cg_image
}

// TODO: should have UIImageOrientation type
- (NSInteger)imageOrientation {
    // FIXME: load image orientation info from file?
    0 // UIImageOrientationUp
}

- (CGSize)size {
    let image = env.objc.borrow::<UIImageHostObject>(this).cg_image;
    let (width, height) = cg_image::borrow_image(&env.objc, image).dimensions();
    CGSize {
        width: width as _,
        height: height as _,
    }
}

- (CGFloat)scale {
    // TODO: support other scales, such as @2x
    1.0
}

- (())drawInRect:(CGRect)rect {
    let context = UIGraphicsGetCurrentContext(env);
    let image = env.objc.borrow::<UIImageHostObject>(this).cg_image;
    CGContextDrawImage(env, context, rect, image);
}

- (())drawAtPoint:(CGPoint)point {
    let context = UIGraphicsGetCurrentContext(env);
    if context == nil {
        log!("Warning: [(UIImage*){:?} drawAtPoint:{:?}] is called with nil context, ignoring.", this, point);
        return;
    }
    let image = env.objc.borrow::<UIImageHostObject>(this).cg_image;
    let rect = CGRect {
        origin: point,
        size: CGSize {
            width: CGImageGetWidth(env, image) as CGFloat,
            height: CGImageGetHeight(env, image) as CGFloat,
        }
    };
    CGContextDrawImage(env, context, rect, image);
}

- (())drawAtPoint:(CGPoint)point
         blendMode:(CGBlendMode)blend_mode
             alpha:(CGFloat)alpha {
    let context = UIGraphicsGetCurrentContext(env);
    if context == nil {
        log!("Warning: [(UIImage*){:?} drawAtPoint:{:?} blendMode:{:?} alpha:{:?}] is called with nil context, ignoring.", this, point, blend_mode, alpha);
        return;
    }
    let image = env.objc.borrow::<UIImageHostObject>(this).cg_image;
    let rect = CGRect {
        origin: point,
        size: CGSize {
            width: CGImageGetWidth(env, image) as CGFloat,
            height: CGImageGetHeight(env, image) as CGFloat,
        }
    };
    CGContextDrawImageWithBlendModeAndAlpha(env, context, rect, image, blend_mode, alpha);
}

- (())drawInRect:(CGRect)rect
        blendMode:(CGBlendMode)blend_mode
            alpha:(CGFloat)alpha {
    let context = UIGraphicsGetCurrentContext(env);
    let image = env.objc.borrow::<UIImageHostObject>(this).cg_image;
    CGContextDrawImageWithBlendModeAndAlpha(env, context, rect, image, blend_mode, alpha);
}

@end

// Undocumented class used in NIBs
// TODO: It's not clear _why_ placeholder is needed?
@implementation UIImageNibPlaceholder: UIImage

// NSCoding implementation
- (id)initWithCoder:(id)coder {
    release(env, this);

    // TODO: decode other attributes
    let key_ns_string = get_static_str(env, "UIResourceName");
    let resource_name: id = msg![env; coder decodeObjectForKey:key_ns_string];

    let res = msg_class![env; UIImage imageNamed:resource_name];
    // TODO: It is not clear if we need to additionally retain here?
    retain(env, res)
}

@end

};

/// PNG CRC-32 (IEEE 802.3), computed without a lookup table.
fn png_crc32(data: &[u8]) -> u32 {
    let mut crc: u32 = 0xFFFF_FFFF;
    for &byte in data {
        crc ^= byte as u32;
        for _ in 0..8 {
            if crc & 1 != 0 {
                crc = (crc >> 1) ^ 0xEDB8_8320;
            } else {
                crc >>= 1;
            }
        }
    }
    crc ^ 0xFFFF_FFFF
}

fn png_write_chunk(out: &mut Vec<u8>, chunk_type: &[u8; 4], data: &[u8]) {
    out.extend_from_slice(&(data.len() as u32).to_be_bytes());
    let mut type_and_data = Vec::with_capacity(4 + data.len());
    type_and_data.extend_from_slice(chunk_type);
    type_and_data.extend_from_slice(data);
    out.extend_from_slice(chunk_type);
    out.extend_from_slice(data);
    out.extend_from_slice(&png_crc32(&type_and_data).to_be_bytes());
}

/// Minimal PNG encoder for 8-bit RGBA. `rgba` must be `width * height * 4`
/// bytes, top-to-bottom rows, with *straight* (non-premultiplied) alpha.
fn encode_png_rgba(rgba: &[u8], width: u32, height: u32) -> Vec<u8> {
    use flate2::write::ZlibEncoder;
    use flate2::Compression;
    use std::io::Write;

    let mut out = Vec::new();
    out.extend_from_slice(&[137, 80, 78, 71, 13, 10, 26, 10]); // signature

    // IHDR: 8-bit depth, color type 6 (truecolour with alpha), no interlace.
    let mut ihdr = Vec::with_capacity(13);
    ihdr.extend_from_slice(&width.to_be_bytes());
    ihdr.extend_from_slice(&height.to_be_bytes());
    ihdr.push(8);
    ihdr.push(6);
    ihdr.push(0);
    ihdr.push(0);
    ihdr.push(0);
    png_write_chunk(&mut out, b"IHDR", &ihdr);

    // IDAT: each scanline prefixed with a filter-type byte (0 = None), then
    // the whole thing zlib-compressed.
    let row_bytes = (width * 4) as usize;
    let mut raw = Vec::with_capacity((row_bytes + 1) * height as usize);
    for y in 0..height as usize {
        raw.push(0);
        raw.extend_from_slice(&rgba[y * row_bytes..(y + 1) * row_bytes]);
    }
    let mut encoder = ZlibEncoder::new(Vec::new(), Compression::default());
    encoder.write_all(&raw).unwrap();
    let compressed = encoder.finish().unwrap();
    png_write_chunk(&mut out, b"IDAT", &compressed);

    png_write_chunk(&mut out, b"IEND", &[]);
    out
}

/// `UIImagePNGRepresentation(UIImage *image)` -> `NSData *`
fn UIImagePNGRepresentation(env: &mut Environment, image: id) -> id {
    if image == nil {
        return nil;
    }
    let cg_image = env.objc.borrow::<UIImageHostObject>(image).cg_image;
    if cg_image == nil {
        return nil;
    }

    // Read pixels back. Image::pixels() is 8-bit RGBA with *premultiplied*
    // alpha; PNG expects straight alpha, so unpremultiply first.
    let (width, height, straight) = {
        let img = cg_image::borrow_image(&env.objc, cg_image);
        let (w, h) = img.dimensions();
        let premul = img.pixels();
        let mut straight = Vec::with_capacity(premul.len());
        let mut i = 0;
        while i < premul.len() {
            let a = premul[i + 3];
            if a == 0 {
                straight.extend_from_slice(&[0, 0, 0, 0]);
            } else {
                let unpre = |c: u8| ((c as u32 * 255 + (a as u32 / 2)) / a as u32).min(255) as u8;
                straight.push(unpre(premul[i]));
                straight.push(unpre(premul[i + 1]));
                straight.push(unpre(premul[i + 2]));
                straight.push(a);
            }
            i += 4;
        }
        (w, h, straight)
    };

    let png = encode_png_rgba(&straight, width, height);

    // Hand the bytes to a fresh NSData (copying into guest memory).
    let length: NSUInteger = png.len().try_into().unwrap();
    let alloc: MutVoidPtr = env.mem.alloc(length);
    env.mem
        .bytes_at_mut(alloc.cast(), length)
        .copy_from_slice(&png);
    let data: id = msg_class![env; NSData dataWithBytesNoCopy:alloc
                                                       length:length];
    data
}

pub const FUNCTIONS: FunctionExports = &[export_c_func!(UIImagePNGRepresentation(_))];
