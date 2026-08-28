/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */
//! `NSNetServiceBrowser`.
//!
//! touchHLE has no Bonjour/mDNS support, so this is a stub: searches never find
//! any services (no delegate callbacks are made), which apps treat as "no local
//! peers available" and continue in single-player.

use crate::objc::{id, objc_classes, ClassExports, HostObject, NSZonePtr};

#[derive(Default)]
struct NSNetServiceBrowserHostObject {
    delegate: id,
}
impl HostObject for NSNetServiceBrowserHostObject {}

pub const CLASSES: ClassExports = objc_classes! {

(env, this, _cmd);

@implementation NSNetServiceBrowser: NSObject

+ (id)allocWithZone:(NSZonePtr)_zone {
    let host_object = Box::<NSNetServiceBrowserHostObject>::default();
    env.objc.alloc_object(this, host_object, &mut env.mem)
}

- (id)init {
    this
}

- (())setDelegate:(id)delegate {
    env.objc.borrow_mut::<NSNetServiceBrowserHostObject>(this).delegate = delegate;
}
- (id)delegate {
    env.objc.borrow::<NSNetServiceBrowserHostObject>(this).delegate
}

- (())searchForServicesOfType:(id)_type // NSString*
                     inDomain:(id)_domain { // NSString*
    // No mDNS: the search simply never reports any services.
}
- (())searchForBrowsableDomains {
}
- (())searchForRegistrationDomains {
}
- (())stop {
    // Nothing is running.
}
- (())scheduleInRunLoop:(id)_run_loop forMode:(id)_mode {
}
- (())removeFromRunLoop:(id)_run_loop forMode:(id)_mode {
}

- (())dealloc {
    env.objc.dealloc_object(this, &mut env.mem)
}

@end

};
