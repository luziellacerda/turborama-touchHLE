/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */
//! `NSNumberFormatter`.

use crate::frameworks::foundation::{ns_string, NSInteger, NSUInteger};
use crate::objc::{autorelease, id, msg, objc_classes, ClassExports, HostObject, NSZonePtr};

// NSNumberFormatterStyle values. touchHLE only distinguishes "has grouping"
// (decimal style) from "no grouping" (none) for now.
type NSNumberFormatterStyle = NSUInteger;
const NSNumberFormatterNoStyle: NSNumberFormatterStyle = 0;
const NSNumberFormatterDecimalStyle: NSNumberFormatterStyle = 1;

struct NSNumberFormatterHostObject {
    number_style: NSNumberFormatterStyle,
    uses_grouping_separator: bool,
    grouping_separator: Option<id>,
}
impl HostObject for NSNumberFormatterHostObject {}

pub const CLASSES: ClassExports = objc_classes! {

(env, this, _cmd);

@implementation NSNumberFormatter: NSObject

+ (id)allocWithZone:(NSZonePtr)_zone {
    let host_object = Box::new(NSNumberFormatterHostObject {
        number_style: NSNumberFormatterNoStyle,
        uses_grouping_separator: false,
        grouping_separator: None,
    });
    env.objc.alloc_object(this, host_object, &mut env.mem)
}

- (())setNumberStyle:(NSNumberFormatterStyle)style {
    let host = env.objc.borrow_mut::<NSNumberFormatterHostObject>(this);
    host.number_style = style;
    // The decimal style groups by default.
    if style == NSNumberFormatterDecimalStyle {
        host.uses_grouping_separator = true;
    }
}
- (NSNumberFormatterStyle)numberStyle {
    env.objc.borrow::<NSNumberFormatterHostObject>(this).number_style
}

- (())setUsesGroupingSeparator:(bool)uses {
    env.objc.borrow_mut::<NSNumberFormatterHostObject>(this).uses_grouping_separator = uses;
}
- (bool)usesGroupingSeparator {
    env.objc.borrow::<NSNumberFormatterHostObject>(this).uses_grouping_separator
}

- (())setGroupingSeparator:(id)separator { // NSString*
    let copy: id = msg![env; separator copy];
    let host = env.objc.borrow_mut::<NSNumberFormatterHostObject>(this);
    if let Some(old) = host.grouping_separator.replace(copy) {
        () = msg![env; old release];
    }
}

// These are accepted but ignored; touchHLE's formatting is minimal.
- (())setMinimumFractionDigits:(NSInteger)_digits {
}
- (())setMaximumFractionDigits:(NSInteger)_digits {
}
- (())setGroupingSize:(NSUInteger)_size {
}
- (())setLocale:(id)_locale {
}
- (())setPositiveFormat:(id)_format { // NSString*
    // touchHLE's minimal formatter doesn't apply a custom format pattern.
}
- (())setNegativeFormat:(id)_format { // NSString*
}
- (())setFormat:(id)_format { // NSString*
}
- (())setFormatterBehavior:(NSInteger)_behavior {
}
- (())setDecimalSeparator:(id)_separator { // NSString*
}
- (())setPaddingCharacter:(id)_character { // NSString*
}
- (())setThousandSeparator:(id)_separator { // NSString*
}
- (())setAllowsFloats:(bool)_allows_floats {
}
- (())setRoundingMode:(NSUInteger)_mode {
}
- (())setMinimumIntegerDigits:(NSInteger)_digits {
}
- (())setPositiveFormat:(id)_format { // NSString*
    // touchHLE's minimal formatter doesn't apply a custom format pattern.
}

- (id)stringFromNumber:(id)number {
    let uses_grouping = env
        .objc
        .borrow::<NSNumberFormatterHostObject>(this)
        .uses_grouping_separator;
    let grouping_separator = env
        .objc
        .borrow::<NSNumberFormatterHostObject>(this)
        .grouping_separator;

    let value: f64 = msg![env; number doubleValue];

    // Render as an integer if the value has no fractional part (the common case
    // for scores), otherwise fall back to a plain float rendering.
    let mut result = if value.fract() == 0.0 {
        let int_value = value as i64;
        if uses_grouping {
            let sep = if let Some(sep_str) = grouping_separator {
                ns_string::to_rust_string(env, sep_str).to_string()
            } else {
                ",".to_string()
            };
            group_thousands(int_value, &sep)
        } else {
            format!("{}", int_value)
        }
    } else {
        format!("{}", value)
    };

    if result.is_empty() {
        result = "0".to_string();
    }

    let res = ns_string::from_rust_string(env, result);
    autorelease(env, res)
}

@end

};

/// Format an integer with a grouping separator every three digits.
fn group_thousands(value: i64, separator: &str) -> String {
    let negative = value < 0;
    let digits = value.unsigned_abs().to_string();
    let bytes = digits.as_bytes();
    let mut grouped = String::new();
    let len = bytes.len();
    for (i, b) in bytes.iter().enumerate() {
        if i != 0 && (len - i) % 3 == 0 {
            grouped.push_str(separator);
        }
        grouped.push(*b as char);
    }
    if negative {
        format!("-{}", grouped)
    } else {
        grouped
    }
}
