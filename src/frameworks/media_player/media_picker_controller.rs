/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */
//! `MPMediaPickerController`.

use crate::frameworks::foundation::NSUInteger;
use crate::objc::{id, msg, objc_classes, ClassExports};

pub const CLASSES: ClassExports = objc_classes! {

(env, this, _cmd);

@implementation MPMediaPickerController: UIViewController

// The picker lets the user choose from their iPod music library. touchHLE has
// no such library, so this is a stub: it returns a valid, initialized (but
// non-functional) controller instead of crashing. Apps that use this for an
// optional "pick your own music" feature can still run; they just won't get a
// working picker.
- (id)initWithMediaTypes:(NSUInteger)_media_types { // MPMediaType
    msg![env; this init]
}

// Configuration setters on the (non-functional) picker. Accepted and ignored.
- (())setAllowsPickingMultipleItems:(bool)_allows {
    // no-op
}
- (())setDelegate:(id)_delegate {
    // no-op
}
- (())setPrompt:(id)_prompt { // NSString*
    // no-op
}

@end

};
