/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */
//! `NSURLConnection`.

use super::ns_run_loop::add_perform_request;
use super::{ns_string, NSInteger};
use crate::environment::Environment;
use crate::mem::MutPtr;
use crate::objc::{
    autorelease, id, msg, msg_class, nil, objc_classes, release, retain, ClassExports, HostObject,
    NSZonePtr,
};
use std::borrow::Cow;

struct NSURLConnectionHostObject {
    delegate: id,
    error: id,
}
impl HostObject for NSURLConnectionHostObject {}

const NSURLErrorDomain: &str = "NSURLErrorDomain";

/// Our helper type, Foundation just uses ints.
type NSURLErrorCode = NSInteger;
const NSURLErrorNotConnectedToInternet: NSURLErrorCode = -1009;

pub const CLASSES: ClassExports = objc_classes! {

(env, this, _cmd);

@implementation NSURLConnection: NSObject

+ (id)allocWithZone:(NSZonePtr)_zone {
    let host_object = Box::new(NSURLConnectionHostObject {
        delegate: nil,
        error: nil,
    });
    env.objc.alloc_object(this, host_object, &mut env.mem)
}

+ (id)sendSynchronousRequest:(id)request // NSURLRequest *
           returningResponse:(MutPtr<id>)response // NSURLResponse **
                       error:(MutPtr<id>)out_error { // NSError **
    log!(
        "TODO: [NSURLConnection sendSynchronousRequest:{:?} ('{}') response:{:?} error:{:?}] -> nil",
        request,
        url_string_from_request(env, request),
        response,
        out_error,
    );
    if !response.is_null() {
        env.mem.write(response, nil);
    }
    if !out_error.is_null() {
        let domain = ns_string::get_static_str(env, NSURLErrorDomain);
        let error = msg_class![env; NSError alloc];
        // TODO: fill userInfo
        let error = msg![env; error initWithDomain:domain code:NSURLErrorNotConnectedToInternet userInfo:nil];
        autorelease(env, error);
        env.mem.write(out_error, error);
    }
    nil
}

+ (id)connectionWithRequest:(id)request // NSURLRequest *
                   delegate:(id)delegate {
    let new: id = msg![env; this alloc];
    let new: id = msg![env; new initWithRequest:request delegate:delegate];
    autorelease(env, new)
}

- (id)initWithRequest:(id)request // NSURLRequest *
             delegate:(id)delegate {
    msg![env; this initWithRequest:request delegate:delegate startImmediately:true]
}

- (id)initWithRequest:(id)request // NSURLRequest *
             delegate:(id)delegate
     startImmediately:(bool)start_immediately {
    log!(
        "[(NSURLConnection *){:?} initWithRequest:{:?} ('{}') delegate:{:?} startImmediately:{}]: no network access, connection will fail with an error",
        this,
        request,
        url_string_from_request(env, request),
        delegate,
        start_immediately,
    );

    // touchHLE does not have network access (unless explicitly allowed, which
    // is not implemented for NSURLConnection yet). Behave like a device
    // without an Internet connection: asynchronously deliver
    // connection:didFailWithError: to the delegate. Many apps only proceed
    // (e.g. past a loading screen) once the connection reports either success
    // or failure, so never delivering anything makes them hang forever.
    let domain = ns_string::get_static_str(env, NSURLErrorDomain);
    let error: id = msg_class![env; NSError alloc];
    // TODO: fill userInfo
    let error: id =
        msg![env; error initWithDomain:domain code:NSURLErrorNotConnectedToInternet userInfo:nil];

    retain(env, delegate);
    let host_object = env
        .objc
        .borrow_mut::<NSURLConnectionHostObject>(this);
    host_object.delegate = delegate;
    host_object.error = error;

    // Deliver on a later run loop iteration, not synchronously: the app has
    // usually not finished setting up its state when this returns.
    //
    // It has to go on the run loop of the thread that started the connection,
    // which is what Apple does by default, and not on the main one. Apps
    // commonly start a connection on a worker thread and then park that thread
    // in `runMode:beforeDate:` waiting for the callback; delivering elsewhere
    // leaves it waiting forever.
    let selector = env
        .objc
        .lookup_selector("_touchHLE_deliverConnectionFailure")
        .unwrap();
    let run_loop: id = msg_class![env; NSRunLoop currentRunLoop];
    add_perform_request(env, run_loop, this, selector, nil, Some(0.0), false);

    this
}

// Private helper: delivers the failure scheduled by
// initWithRequest:delegate:startImmediately: (see above). Needed because the
// run loop's perform-request machinery only supports single-argument
// selectors.
- (())_touchHLE_deliverConnectionFailure {
    let &NSURLConnectionHostObject { delegate, error } = env.objc.borrow(this);
    let responds: bool = match env.objc.lookup_selector("connection:didFailWithError:") {
        Some(sel) => msg![env; delegate respondsToSelector:sel],
        None => false,
    };
    if responds {
        let _: () = msg![env; delegate connection:this didFailWithError:error];
    } else {
        log!(
            "Warning: NSURLConnection delegate {:?} does not respond to connection:didFailWithError:, dropping error",
            delegate
        );
    }
    let host_object = env
        .objc
        .borrow_mut::<NSURLConnectionHostObject>(this);
    host_object.delegate = nil;
    host_object.error = nil;
    release(env, error);
    release(env, delegate);
}

- (())cancel {
    // We never deliver anything after the failure, so there is nothing to do.
    log_dbg!("[(NSURLConnection *){:?} cancel]", this);
}

@end

};

fn url_string_from_request(env: &mut Environment, request: id) -> Cow<'static, str> {
    if request == nil {
        Cow::from("(null)")
    } else {
        let url = msg![env; request URL];
        let ns_string = msg![env; url absoluteString];
        ns_string::to_rust_string(env, ns_string)
    }
}
