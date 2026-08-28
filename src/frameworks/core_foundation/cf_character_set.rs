/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */
//! `CFCharacterSet`
//!
//! This is toll-free bridged to `NSCharacterSet` in Apple's implementation;
//! here they are the same type, so the predefined-set getter just forwards to
//! the corresponding `NSCharacterSet` class method.

use super::CFTypeRef;
use crate::dyld::{export_c_func, FunctionExports};
use crate::frameworks::foundation::{unichar, NSUInteger};
use crate::objc::{msg, msg_class};
use crate::Environment;

/// `NSCharacterSet` is toll-free bridged with `CFCharacterSetRef`.
pub type CFCharacterSetRef = CFTypeRef;

// Values of the CFCharacterSetPredefinedSet enum (see CFCharacterSet.h).
type CFCharacterSetPredefinedSet = NSUInteger;
const kCFCharacterSetControl: CFCharacterSetPredefinedSet = 1;
const kCFCharacterSetWhitespace: CFCharacterSetPredefinedSet = 2;
const kCFCharacterSetWhitespaceAndNewline: CFCharacterSetPredefinedSet = 3;
const kCFCharacterSetDecimalDigit: CFCharacterSetPredefinedSet = 4;
const kCFCharacterSetPunctuation: CFCharacterSetPredefinedSet = 11;
const kCFCharacterSetLetter: CFCharacterSetPredefinedSet = 5;
const kCFCharacterSetLowercaseLetter: CFCharacterSetPredefinedSet = 6;
const kCFCharacterSetUppercaseLetter: CFCharacterSetPredefinedSet = 7;
const kCFCharacterSetNewline: CFCharacterSetPredefinedSet = 15;

fn CFCharacterSetGetPredefined(
    env: &mut Environment,
    the_set_identifier: CFCharacterSetPredefinedSet,
) -> CFCharacterSetRef {
    // Forward to the toll-free-bridged NSCharacterSet class method for the sets
    // touchHLE implements. These return shared, autoreleased instances, which
    // matches CFCharacterSetGetPredefined returning a non-owned reference.
    match the_set_identifier {
        kCFCharacterSetControl => msg_class![env; NSCharacterSet controlCharacterSet],
        kCFCharacterSetWhitespace => msg_class![env; NSCharacterSet whitespaceCharacterSet],
        kCFCharacterSetWhitespaceAndNewline => {
            msg_class![env; NSCharacterSet whitespaceAndNewlineCharacterSet]
        }
        kCFCharacterSetDecimalDigit => msg_class![env; NSCharacterSet decimalDigitCharacterSet],
        kCFCharacterSetPunctuation => msg_class![env; NSCharacterSet punctuationCharacterSet],
        kCFCharacterSetLetter => msg_class![env; NSCharacterSet letterCharacterSet],
        kCFCharacterSetLowercaseLetter => {
            msg_class![env; NSCharacterSet lowercaseLetterCharacterSet]
        }
        kCFCharacterSetUppercaseLetter => {
            msg_class![env; NSCharacterSet uppercaseLetterCharacterSet]
        }
        kCFCharacterSetNewline => msg_class![env; NSCharacterSet newlineCharacterSet],
        _ => todo!(
            "CFCharacterSetGetPredefined: unimplemented predefined set {}",
            the_set_identifier
        ),
    }
}

fn CFCharacterSetIsCharacterMember(
    env: &mut Environment,
    the_set: CFCharacterSetRef,
    character: unichar,
) -> bool {
    // Toll-free bridged to NSCharacterSet's -characterIsMember:.
    msg![env; the_set characterIsMember:character]
}

pub const FUNCTIONS: FunctionExports = &[
    export_c_func!(CFCharacterSetGetPredefined(_)),
    export_c_func!(CFCharacterSetIsCharacterMember(_, _)),
];
