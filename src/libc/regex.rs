/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */
//! `regex.h` (POSIX regular expressions)
//!
//! Backed by the host `regex-lite` crate. POSIX BRE patterns are translated
//! to modern (ERE-like) syntax before compilation. The guest's `regex_t` is
//! treated as an opaque handle: the compiled regex lives host-side, keyed by
//! the guest pointer.

use crate::dyld::{export_c_func, FunctionExports};
use crate::mem::{ConstPtr, GuestUSize, MutPtr, Ptr, SafeRead};
use crate::Environment;
use regex_lite::Regex;
use std::collections::HashMap;

// cflags (Darwin values)
const REG_EXTENDED: i32 = 0o0001;
const REG_ICASE: i32 = 0o0002;
const REG_NOSUB: i32 = 0o0004;
const REG_NEWLINE: i32 = 0o0010;
const REG_NOSPEC: i32 = 0o0020;
const REG_PEND: i32 = 0o0040;

// eflags
const REG_NOTBOL: i32 = 0o0001;
const REG_NOTEOL: i32 = 0o0002;
const REG_STARTEND: i32 = 0o0004;

// error codes
const REG_NOMATCH: i32 = 1;
const REG_BADPAT: i32 = 2;
const REG_ESUBREG: i32 = 6;
const REG_ESPACE: i32 = 12;

/// Magic value used by the BSD implementation ("re" with high bit tricks);
/// we only need something recognisably non-zero.
const RE_MAGIC: i32 = ((b'r' as i32 ^ 0o200) << 8) | b'e' as i32;

#[allow(non_camel_case_types)]
#[repr(C, packed)]
struct regex_t {
    re_magic: i32,
    re_nsub: GuestUSize,
    /// `const char *`, only used with REG_PEND (unsupported)
    re_endp: ConstPtr<u8>,
    /// `struct re_guts *`, opaque; unused by us
    re_g: MutPtr<u8>,
}
unsafe impl SafeRead for regex_t {}

/// `regoff_t` is `off_t` on Darwin, i.e. 64-bit even on 32-bit ARM.
#[allow(non_camel_case_types)]
#[repr(C, packed)]
struct regmatch_t {
    rm_so: i64,
    rm_eo: i64,
}
unsafe impl SafeRead for regmatch_t {}

struct CompiledRegex {
    regex: Regex,
    nosub: bool,
}

#[derive(Default)]
pub struct State {
    /// Compiled regexes, keyed by the guest address of the `regex_t`.
    regexes: HashMap<u32, CompiledRegex>,
}

/// Escape a single character so it is matched literally.
fn push_literal(out: &mut String, c: char) {
    if c.is_ascii_alphanumeric() {
        out.push(c);
    } else {
        out.push('\\');
        out.push(c);
    }
}

/// Translate a POSIX pattern to the syntax understood by `regex-lite`.
/// ERE is already close; BRE needs the roles of escaped/unescaped
/// `()`/`{}` swapped and `+?|` made literal.
fn translate_pattern(pattern: &str, extended: bool) -> Result<String, i32> {
    let mut out = String::with_capacity(pattern.len() + 8);
    let mut chars = pattern.chars().peekable();
    let mut in_bracket = false;
    while let Some(c) = chars.next() {
        if in_bracket {
            // Bracket expressions pass through unchanged.
            out.push(c);
            if c == ']' {
                in_bracket = false;
            }
            continue;
        }
        match c {
            '[' => {
                in_bracket = true;
                out.push(c);
                // ']' or '^]' directly after '[' is a literal ']'
                if chars.peek() == Some(&'^') {
                    out.push(chars.next().unwrap());
                }
                if chars.peek() == Some(&']') {
                    out.push('\\');
                    out.push(chars.next().unwrap());
                }
            }
            '\\' => {
                let Some(next) = chars.next() else {
                    return Err(REG_BADPAT); // trailing backslash
                };
                match next {
                    // BRE: \( \) \{ \} are the operators
                    '(' | ')' | '{' | '}' if !extended => out.push(next),
                    // Backreferences are not supported by regex-lite
                    '1'..='9' => return Err(REG_ESUBREG),
                    // GNU extensions \+ \? \| in BRE mean the operators
                    '+' | '?' | '|' if !extended => out.push(next),
                    _ => push_literal(&mut out, next),
                }
            }
            // BRE: unescaped ( ) { } + ? | are literals
            '(' | ')' | '{' | '}' | '+' | '?' | '|' if !extended => {
                out.push('\\');
                out.push(c);
            }
            _ => out.push(c),
        }
    }
    if in_bracket {
        return Err(REG_BADPAT);
    }
    Ok(out)
}

fn regcomp(
    env: &mut Environment,
    preg: MutPtr<regex_t>,
    pattern: ConstPtr<u8>,
    cflags: i32,
) -> i32 {
    let Ok(pattern_str) = env.mem.cstr_at_utf8(pattern) else {
        log!("Warning: regcomp() with non-UTF-8 pattern, returning REG_BADPAT");
        return REG_BADPAT;
    };
    let pattern_str = pattern_str.to_string();
    log_dbg!("regcomp({:?}, {:?}, {:#o})", preg, pattern_str, cflags);

    if cflags & REG_PEND != 0 {
        log!("Warning: regcomp() with unsupported REG_PEND flag (ignored)");
    }

    let translated = if cflags & REG_NOSPEC != 0 {
        // Literal string matching
        let mut out = String::with_capacity(pattern_str.len() * 2);
        for c in pattern_str.chars() {
            push_literal(&mut out, c);
        }
        out
    } else {
        match translate_pattern(&pattern_str, cflags & REG_EXTENDED != 0) {
            Ok(t) => t,
            Err(code) => return code,
        }
    };

    // Inline flags: (?i) case-insensitive; POSIX '.' matches newline unless
    // REG_NEWLINE is given, which also makes ^/$ match at line boundaries.
    let mut prefix = String::new();
    if cflags & REG_ICASE != 0 {
        prefix.push_str("(?i)");
    }
    if cflags & REG_NEWLINE != 0 {
        prefix.push_str("(?m)");
    } else {
        prefix.push_str("(?s)");
    }
    let full_pattern = format!("{}{}", prefix, translated);

    let regex = match Regex::new(&full_pattern) {
        Ok(r) => r,
        Err(e) => {
            log!(
                "Warning: regcomp() failed to compile {:?} (translated: {:?}): {}",
                pattern_str,
                full_pattern,
                e
            );
            return REG_BADPAT;
        }
    };

    let nsub: GuestUSize = (regex.captures_len() - 1).try_into().unwrap();
    env.mem.write(
        preg,
        regex_t {
            re_magic: RE_MAGIC,
            re_nsub: nsub,
            re_endp: Ptr::null(),
            re_g: Ptr::null(),
        },
    );

    env.libc_state.regex.regexes.insert(
        preg.to_bits(),
        CompiledRegex {
            regex,
            nosub: cflags & REG_NOSUB != 0,
        },
    );
    0 // success
}

fn regexec(
    env: &mut Environment,
    preg: ConstPtr<regex_t>,
    string: ConstPtr<u8>,
    nmatch: GuestUSize,
    pmatch: MutPtr<regmatch_t>,
    eflags: i32,
) -> i32 {
    let Some(compiled) = env.libc_state.regex.regexes.get(&preg.to_bits()) else {
        log!(
            "Warning: regexec() with unknown regex_t {:?} (moved or never compiled?)",
            preg
        );
        return REG_BADPAT;
    };

    if eflags & (REG_NOTBOL | REG_NOTEOL | REG_STARTEND) != 0 {
        log!(
            "Warning: regexec() with unsupported eflags {:#o} (ignored)",
            eflags
        );
    }

    let bytes = env.mem.cstr_at(string);
    let haystack = String::from_utf8_lossy(bytes).into_owned();
    log_dbg!("regexec({:?}, {:?}, {})", preg, haystack, nmatch);

    if compiled.nosub || nmatch == 0 || pmatch.is_null() {
        return if compiled.regex.is_match(&haystack) {
            0
        } else {
            REG_NOMATCH
        };
    }

    let Some(captures) = compiled.regex.captures(&haystack) else {
        return REG_NOMATCH;
    };
    for i in 0..nmatch {
        let (rm_so, rm_eo) = captures
            .get(i as usize)
            .map_or((-1, -1), |m| (m.start() as i64, m.end() as i64));
        env.mem.write(pmatch + i, regmatch_t { rm_so, rm_eo });
    }
    0 // success
}

fn regfree(env: &mut Environment, preg: MutPtr<regex_t>) {
    log_dbg!("regfree({:?})", preg);
    if env
        .libc_state
        .regex
        .regexes
        .remove(&preg.to_bits())
        .is_none()
    {
        log!("Warning: regfree() with unknown regex_t {:?}", preg);
        return;
    }
    let mut re = env.mem.read(preg);
    re.re_magic = 0;
    env.mem.write(preg, re);
}

fn regerror(
    env: &mut Environment,
    errcode: i32,
    _preg: ConstPtr<regex_t>,
    errbuf: MutPtr<u8>,
    errbuf_size: GuestUSize,
) -> GuestUSize {
    let msg: &str = match errcode {
        0 => "success",
        REG_NOMATCH => "regexec() failed to match",
        REG_BADPAT => "invalid regular expression",
        3 => "invalid collating element",
        4 => "invalid character class",
        5 => "trailing backslash (\\)",
        REG_ESUBREG => "invalid backreference number",
        7 => "brackets ([ ]) not balanced",
        8 => "parentheses not balanced",
        9 => "braces not balanced",
        10 => "invalid repetition count(s)",
        11 => "invalid character range",
        REG_ESPACE => "out of memory",
        13 => "repetition-operator operand invalid",
        _ => "unknown regex error",
    };
    let needed: GuestUSize = (msg.len() + 1).try_into().unwrap();
    if !errbuf.is_null() && errbuf_size > 0 {
        let copy_len = msg.len().min(errbuf_size as usize - 1);
        for (i, byte) in msg.as_bytes()[..copy_len].iter().enumerate() {
            env.mem.write(errbuf + i as GuestUSize, *byte);
        }
        env.mem.write(errbuf + copy_len as GuestUSize, b'\0');
    }
    needed
}

pub const FUNCTIONS: FunctionExports = &[
    export_c_func!(regcomp(_, _, _)),
    export_c_func!(regexec(_, _, _, _, _)),
    export_c_func!(regfree(_)),
    export_c_func!(regerror(_, _, _, _)),
];

#[cfg(test)]
mod tests {
    use super::translate_pattern;

    #[test]
    fn test_translate_bre() {
        // BRE: \( \) are groups, bare ( ) are literals
        assert_eq!(
            translate_pattern(r"\([a-z]*\)=(x)", false).unwrap(),
            r"([a-z]*)=\(x\)"
        );
        // + and ? are literals in BRE
        assert_eq!(translate_pattern(r"a+b?", false).unwrap(), r"a\+b\?");
        // bracket expressions pass through
        assert_eq!(translate_pattern(r"[+?()]", false).unwrap(), r"[+?()]");
    }

    #[test]
    fn test_translate_ere() {
        assert_eq!(
            translate_pattern(r"([a-z]+)=([0-9]+)", true).unwrap(),
            r"([a-z]+)=([0-9]+)"
        );
    }
}
