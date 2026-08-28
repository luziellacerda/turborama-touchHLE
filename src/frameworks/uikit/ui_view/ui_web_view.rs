/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */
//! `UIWebView`.

use crate::frameworks::foundation::ns_string::{get_static_str, to_rust_string};
use crate::msg;
use crate::objc::{
    id, impl_HostObject_with_superclass, msg_super, nil, objc_classes, ClassExports, NSZonePtr,
};
use crate::Environment;
use std::borrow::Cow;

pub struct UIWebViewHostObject {
    superclass: super::UIViewHostObject,
    /// Not retained, matching UIKit's assign semantics for this delegate.
    delegate: id,
}
impl_HostObject_with_superclass!(UIWebViewHostObject);
impl Default for UIWebViewHostObject {
    fn default() -> Self {
        UIWebViewHostObject {
            superclass: Default::default(),
            delegate: nil,
        }
    }
}

/// The load methods below don't actually load anything, so no
/// `webViewDidFinishLoad:` callback would ever arrive on its own, and apps
/// that wait for one (ad frameworks in particular) would wait forever.
/// Pretend the load succeeded instead: if the delegate implements
/// `webViewDidFinishLoad:`, schedule it for the next run loop iteration,
/// which matches the asynchronous behaviour of the real UIKit.
fn schedule_did_finish_load(env: &mut Environment, this: id) {
    let delegate = env.objc.borrow::<UIWebViewHostObject>(this).delegate;
    if delegate == nil {
        return;
    }
    let sel = env
        .objc
        .register_host_selector("webViewDidFinishLoad:".to_string(), &mut env.mem);
    if !env.objc.object_has_method(&env.mem, delegate, sel) {
        return;
    }
    () = msg![env; delegate performSelector:sel withObject:this afterDelay:0.0];
}

pub const CLASSES: ClassExports = objc_classes! {

(env, this, _cmd);

@implementation UIWebView: UIView

+ (id)allocWithZone:(NSZonePtr)_zone {
    let host_object = Box::<UIWebViewHostObject>::default();
    env.objc.alloc_object(this, host_object, &mut env.mem)
}

// NSCoding implementation
- (id)initWithCoder:(id)coder {
    // The web view isn't actually rendered, so there are no web-view-specific
    // archived properties to decode; UIView's initWithCoder: handles the
    // standard view geometry from the NIB.
    msg_super![env; this initWithCoder:coder]
}

- (())setScalesPageToFit:(bool)_scales {
    // TODO
}
- (())setDelegate:(id)delegate {
    env.objc.borrow_mut::<UIWebViewHostObject>(this).delegate = delegate;
}
- (())setDetectsPhoneNumbers:(bool)_detects {
    // TODO
}
- (id)delegate {
    env.objc.borrow::<UIWebViewHostObject>(this).delegate
}
- (())loadRequest:(id)request { // NSURLRequest*
    let url_string = if request != nil {
        let url = msg![env; request URL];
        let url_desc = msg![env; url description];
        to_rust_string(env, url_desc)
    } else {
        Cow::default()
    };
    log!("TODO: [(UIWebView*) {:?} loadRequest:{:?} ({})]", this, request, url_string);
    schedule_did_finish_load(env, this);
}
- (())loadHTMLString:(id)string // NSString*
             baseURL:(id)base_url { // NSURL*
    log!(
        "TODO: [(UIWebView*) {:?} loadHTMLString:{:?} baseURL:{:?}]",
        this, string, base_url
    );
    schedule_did_finish_load(env, this);
}
- (id)stringByEvaluatingJavaScriptFromString:(id)script { // NSString*
    let script_string = if script != nil {
        to_rust_string(env, script)
    } else {
        Cow::default()
    };
    log!(
        "TODO: [(UIWebView*) {:?} stringByEvaluatingJavaScriptFromString:{:?}] (returning empty string)",
        this, script_string
    );
    get_static_str(env, "")
}
// touchHLE doesn't implement real page navigation (loadRequest:/
// loadHTMLString:baseURL: are no-ops), so there is never any history to
// go back or forward through.
- (bool)canGoBack {
    false
}
- (bool)canForward {
    false
}
// canForward above used the wrong selector name; canGoForward is the
// real UIWebView method.
- (bool)canGoForward {
    false
}
- (())goBack {
    log!("TODO: [(UIWebView*) {:?} goBack] (ignored)", this);
}
- (())goForward {
    log!("TODO: [(UIWebView*) {:?} goForward] (ignored)", this);
}
- (())reload {
    log!("TODO: [(UIWebView*) {:?} reload] (ignored)", this);
}
- (())stopLoading {
    log!("TODO: [(UIWebView*) {:?} stopLoading] (ignored)", this);
}
- (bool)isLoading {
    false
}

@end

};
