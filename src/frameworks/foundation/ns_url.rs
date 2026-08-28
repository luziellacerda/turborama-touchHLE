/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */
//! `NSURL`.

use super::ns_string::{from_rust_string, get_static_str, to_rust_string};
use super::NSUInteger;
use crate::fs::{GuestPath, GuestPathBuf};
use crate::mem::MutPtr;
use crate::objc::{
    autorelease, id, msg, msg_class, nil, objc_classes, release, retain, ClassExports, HostObject,
    NSZonePtr,
};
use crate::Environment;
use std::borrow::Cow;

/// It seems like there's two kinds of NSURLs: ones for file paths, and others.
/// So far only the former is implemented (TODO).
enum NSURLHostObject {
    /// This is a file URL. The NSString is a system path (no `file:///`).
    ///
    /// This is a wrapper around NSString so that conversions between NSURL
    /// and NSString, which happen often, can be simple and efficient.
    FileURL {
        ns_string: id,
        // Relative file URL save the working directory at the time of creation
        // At the moment, used in the description selector.
        working_directory: GuestPathBuf,
    },
    /// Non-file URL.
    OtherURL { ns_string: id },
}
impl HostObject for NSURLHostObject {}

/// Extract the filesystem path from a `file:` URL string, with
/// percent-decoding. Handles the forms `file:///path`,
/// `file://host/path` (host is ignored, like `localhost`) and `file:/path`
/// (RFC 8089). Returns [None] if the string doesn't have a `file:` scheme.
fn file_url_to_path(url_string: &str) -> Option<String> {
    // URL schemes are case-insensitive.
    if url_string.len() < 5 || !url_string[..5].eq_ignore_ascii_case("file:") {
        return None;
    }
    let rest = &url_string[5..];
    let path = if let Some(after_slashes) = rest.strip_prefix("//") {
        // Skip the authority component (e.g. "localhost") up to the next '/'.
        match after_slashes.find('/') {
            Some(idx) => &after_slashes[idx..],
            None => "/",
        }
    } else {
        rest
    };
    // Percent-decoding (e.g. "%20" => " ").
    let bytes = path.as_bytes();
    let mut decoded = Vec::with_capacity(bytes.len());
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'%' && i + 2 < bytes.len() {
            if let Ok(byte) = u8::from_str_radix(&path[i + 1..i + 3], 16) {
                decoded.push(byte);
                i += 3;
                continue;
            }
        }
        decoded.push(bytes[i]);
        i += 1;
    }
    Some(String::from_utf8_lossy(&decoded).into_owned())
}

pub const CLASSES: ClassExports = objc_classes! {

(env, this, _cmd);

@implementation NSURL: NSObject

+ (id)allocWithZone:(NSZonePtr)_zone {
    let host_object = NSURLHostObject::FileURL { ns_string: nil, working_directory: env.fs.working_directory().into() };
    env.objc.alloc_object(this, Box::new(host_object), &mut env.mem)
}

+ (id)URLWithString:(id)url { // NSString*
    let new: id = msg![env; this alloc];
    let new: id = msg![env; new initWithString:url];
    autorelease(env, new)
}

+ (id)fileURLWithPath:(id)path { // NSString*
    let new: id = msg![env; this alloc];
    let new: id = msg![env; new initFileURLWithPath:path];
    autorelease(env, new)
}

+ (id)fileURLWithPath:(id)path // NSString*
          isDirectory:(bool)is_dir {
    let new: id = msg![env; this alloc];
    let new: id = msg![env; new initFileURLWithPath:path isDirectory:is_dir];
    autorelease(env, new)
}

- (())dealloc {
    match *env.objc.borrow(this) {
        NSURLHostObject::FileURL { ns_string, .. } => release(env, ns_string),
        NSURLHostObject::OtherURL { ns_string } => release(env, ns_string),
    }
    env.objc.dealloc_object(this, &mut env.mem)
}

// NSCopying implementation
- (id)copyWithZone:(NSZonePtr)_zone {
    retain(env, this)
}

- (id)initFileURLWithPath:(id)path { // NSString*
    // FIXME: this should guess whether the path is a directory
    msg![env; this initFileURLWithPath:path isDirectory:false]
}

- (id)initFileURLWithPath:(id)path // NSString*
              isDirectory:(bool)_is_dir {
    // FIXME: this does not resolve relative paths to be absolute!
    // Some apps pass file: URL strings where a plain path is expected;
    // strip the scheme and authority in that case.
    let path_string = to_rust_string(env, path).to_string();
    let path = if let Some(stripped) = file_url_to_path(&path_string) {
        let stripped = from_rust_string(env, stripped);
        autorelease(env, stripped)
    } else {
        path
    };
    let path = msg![env; path stringByExpandingTildeInPath];
    let path: id = msg![env; path copy];
    *env.objc.borrow_mut(this) = NSURLHostObject::FileURL { ns_string: path, working_directory: env.fs.working_directory().into() };
    this
}

- (id)initWithString:(id)url { // NSString*
    if url == nil {
        return nil;
    }

    // A file: URL string must become a file URL (see NSURLHostObject),
    // so that isFileURL, path etc. behave correctly.
    let url_string = to_rust_string(env, url).to_string();
    if let Some(path) = file_url_to_path(&url_string) {
        log_dbg!(
            "[(NSURL*){:?} initWithString:{:?}]: parsed file URL {:?} as path {:?}",
            this,
            url,
            url_string,
            path
        );
        let path = from_rust_string(env, path);
        let path = autorelease(env, path);
        return msg![env; this initFileURLWithPath:path];
    }

    // FIXME: this should parse other URL schemes properly
    let url: id = msg![env; url copy];
    *env.objc.borrow_mut(this) = NSURLHostObject::OtherURL { ns_string: url };
    this
}

- (bool)isFileURL {
    match env.objc.borrow(this) {
        NSURLHostObject::FileURL { .. } => true,
        NSURLHostObject::OtherURL { .. } => false,
    }
}

- (id)description {
    match env.objc.borrow(this) {
        NSURLHostObject::FileURL { ns_string, working_directory } => {
            let working_directory = working_directory.as_str().to_string();
            let mut description = to_rust_string(env, *ns_string).to_string().clone();
            if !description.starts_with('/') {
                description = format!("{} -- file://localhost{}", description.trim_start_matches("./"), working_directory );
            }
            let desc = from_rust_string(env, description);
            autorelease(env, desc)
        },
        NSURLHostObject::OtherURL { ns_string } => *ns_string,
    }
}

- (id)path {
    match *env.objc.borrow(this) {
        NSURLHostObject::FileURL { ns_string, .. } => ns_string,
        NSURLHostObject::OtherURL { ns_string } => {
            // TODO: Support full URLs, not only ones that are just a path.
            // FIXME: This should do unescaping.
            // TODO: Avoid copy.
            assert!(to_rust_string(env, ns_string).starts_with('/'));
            ns_string
        },
    }
}

- (id)absoluteString {
    match *env.objc.borrow(this) {
        // FIXME: don't assume URL is already absolute
        NSURLHostObject::FileURL { ns_string, .. } => ns_string,
        NSURLHostObject::OtherURL { ns_string } => {
            // TODO: full RFC 1808 resolution
            assert!(to_rust_string(env, ns_string).starts_with("http"));
            ns_string
        },
    }
}

- (id)absoluteURL {
    let (ns_string, working_directory) = match env.objc.borrow(this) {
        NSURLHostObject::FileURL {
            ns_string,
            working_directory,
        } => (*ns_string, Some(working_directory.as_str().to_string())),
        NSURLHostObject::OtherURL { ns_string } => (*ns_string, None),
    };
    let Some(working_directory) = working_directory else {
        // FIXME: don't assume a non-file URL is already absolute
        return this;
    };
    let path = to_rust_string(env, ns_string).to_string();
    if path.starts_with('/') {
        // Already absolute.
        return this;
    }
    // Resolve the relative path against the working directory saved at
    // creation time.
    let absolute = format!(
        "{}/{}",
        working_directory.trim_end_matches('/'),
        path.trim_start_matches("./")
    );
    let absolute = from_rust_string(env, absolute);
    let absolute = autorelease(env, absolute);
    msg_class![env; NSURL fileURLWithPath:absolute]
}

- (bool)getFileSystemRepresentation:(MutPtr<u8>)buffer
                          maxLength:(NSUInteger)buffer_size {
    let &NSURLHostObject::FileURL { ns_string, .. } = env.objc.borrow(this) else {
        unimplemented!(); // TODO
    };
    msg![env; ns_string getFileSystemRepresentation:buffer maxLength:buffer_size]
}

- (id)URLByAppendingPathComponent:(id)path_component // NSString *
                      isDirectory:(bool)is_directory {
    let &NSURLHostObject::FileURL { ns_string, .. } = env.objc.borrow(this) else {
        unimplemented!(); // TODO
    };
    let mut path: id = msg![env; ns_string stringByAppendingPathComponent:path_component];
    if is_directory {
        path = msg![env; path stringByAppendingString:(get_static_str(env, "/"))];
    }
    msg_class![env; NSURL fileURLWithPath:path]
}

- (id)URLByDeletingLastPathComponent {
    let &NSURLHostObject::FileURL { ns_string, .. } = env.objc.borrow(this) else {
        unimplemented!(); // TODO
    };
    let path: id = msg![env; ns_string stringByDeletingLastPathComponent];
    msg_class![env; NSURL fileURLWithPath:path]
}

// TODO: more constructors, more accessors

@end

// A caching layer a top of NSURL, it's OK to stub
// as we don't have yet a networking support
@implementation NSURLCache: NSObject
+ (id)allocWithZone:(NSZonePtr)_zone {
    // No state: this is a non-functional cache stub.
    let host_object = Box::new(crate::objc::TrivialHostObject);
    env.objc.alloc_object(this, host_object, &mut env.mem)
}
+ (id)sharedURLCache {
    // TODO
    nil
}
- (id)initWithMemoryCapacity:(NSUInteger)_memory_capacity
                diskCapacity:(NSUInteger)_disk_capacity
                    diskPath:(id)_path { // NSString*
    // touchHLE doesn't cache URL responses; accept the request and return a
    // functionless cache object.
    this
}
- (())setMemoryCapacity:(NSUInteger)_memory_capacity {
}
- (())setDiskCapacity:(NSUInteger)_disk_capacity {
}
@end

};

/// Shortcut for host code, provides a view of a URL as a path.
/// TODO: Try to avoid allocating a new GuestPathBuf in more cases.
pub fn to_rust_path(env: &mut Environment, url: id) -> Cow<'static, GuestPath> {
    let path_string: id = msg![env; url path];

    match to_rust_string(env, path_string) {
        Cow::Borrowed(path) => Cow::Borrowed(path.as_ref()),
        Cow::Owned(path_buf) => Cow::Owned(path_buf.into()),
    }
}
