/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */
//! `CFPreferences`.
//!
//! According to Apple's docs, it's not toll-free bridged to `NSUserDefaults`,
//! but we are still implementing one atop of another.

use super::cf_string::CFStringRef;
use super::CFTypeRef;
use crate::dyld::{export_c_func, ConstantExports, FunctionExports, HostConstant};
use crate::frameworks::foundation::ns_string;
use crate::objc::{id, msg, msg_class, nil};
use crate::Environment;

type CFPropertyListRef = CFTypeRef;

/// Whether `app_id` refers to the running app, which is the only one whose
/// preferences touchHLE can serve.
///
/// Apple treats `kCFPreferencesCurrentApplication` and the app's own bundle
/// identifier as equivalent, and apps use both. Any other identifier names a
/// different app, whose preferences are out of reach on a sandboxed iOS anyway.
fn is_current_app(env: &mut Environment, app_id: CFStringRef) -> bool {
    if app_id.is_null() {
        return false;
    }
    let current_app = ns_string::get_static_str(env, kCFPreferencesCurrentApplication);
    if msg![env; app_id isEqualToString:current_app] {
        return true;
    }
    let bundle_id = env.bundle.bundle_identifier().to_string();
    let bundle_id_ns = ns_string::from_rust_string(env, bundle_id);
    let matches: bool = msg![env; app_id isEqualToString:bundle_id_ns];
    if !matches {
        log!(
            "TODO: CFPreferences for a different app ({:?}), treating as empty",
            ns_string::to_rust_string(env, app_id)
        );
    }
    matches
}

fn CFPreferencesCopyAppValue(
    env: &mut Environment,
    key: CFStringRef,
    app_id: CFStringRef,
) -> CFPropertyListRef {
    if !is_current_app(env, app_id) {
        return nil;
    }
    let user_defaults: id = msg_class![env; NSUserDefaults standardUserDefaults];
    let value: id = msg![env; user_defaults objectForKey:key];
    msg![env; value copy]
}

fn CFPreferencesSetAppValue(
    env: &mut Environment,
    key: CFStringRef,
    value: CFPropertyListRef,
    app_id: CFStringRef,
) {
    assert!(!value.is_null()); // TODO
    if !is_current_app(env, app_id) {
        return;
    }
    let user_defaults: id = msg_class![env; NSUserDefaults standardUserDefaults];
    msg![env; user_defaults setObject:value forKey:key]
}

fn CFPreferencesAppSynchronize(env: &mut Environment, app_id: CFStringRef) -> bool {
    if !is_current_app(env, app_id) {
        return false;
    }
    let user_defaults: id = msg_class![env; NSUserDefaults standardUserDefaults];
    msg![env; user_defaults synchronize]
}

pub const kCFPreferencesCurrentApplication: &str = "kCFPreferencesCurrentApplication";

pub const CONSTANTS: ConstantExports = &[(
    "_kCFPreferencesCurrentApplication",
    HostConstant::NSString(kCFPreferencesCurrentApplication),
)];

pub const FUNCTIONS: FunctionExports = &[
    export_c_func!(CFPreferencesCopyAppValue(_, _)),
    export_c_func!(CFPreferencesSetAppValue(_, _, _)),
    export_c_func!(CFPreferencesAppSynchronize(_)),
];
