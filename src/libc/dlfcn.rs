/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */
//! `dlfcn.h` (`dlopen()` and friends)

use crate::dyld::{export_c_func, FunctionExports};
use crate::mem::{ConstPtr, ConstVoidPtr, MutPtr, MutVoidPtr, Ptr, SafeRead};
use crate::Environment;

const RTLD_DEFAULT: MutVoidPtr = Ptr::from_bits(-2 as _);

/// `Dl_info` as filled in by `dladdr()`. Four guest pointers, matching the
/// Darwin/iPhoneOS layout:
/// ```c
/// typedef struct dl_info {
///     const char *dli_fname; // pathname of shared object
///     void       *dli_fbase; // base address of shared object
///     const char *dli_sname; // name of nearest symbol
///     void       *dli_saddr; // address of nearest symbol
/// } Dl_info;
/// ```
#[allow(non_camel_case_types)]
#[repr(C, packed)]
struct Dl_info {
    dli_fname: ConstPtr<u8>,
    dli_fbase: MutVoidPtr,
    dli_sname: ConstPtr<u8>,
    dli_saddr: MutVoidPtr,
}
unsafe impl SafeRead for Dl_info {}

fn is_known_library(path: &str) -> bool {
    crate::dyld::DYLIB_LIST
        .iter()
        .any(|dylib| dylib.path == path || dylib.aliases.contains(&path))
}

fn dlopen(env: &mut Environment, path: ConstPtr<u8>, _mode: i32) -> MutVoidPtr {
    if path.is_null() {
        return RTLD_DEFAULT;
    }
    // TODO: dlopen() support for real dynamic libraries.
    assert!(is_known_library(env.mem.cstr_at_utf8(path).unwrap()));
    // For convenience, use the path as the handle.
    // TODO: Find out whether the handle is truly opaque on iPhone OS, and if
    // not, where it points.
    path.cast_mut().cast()
}

/// Symbols that are known to be legitimately optional: some runtimes (Mono
/// among them) use `dlsym()` specifically to *probe* whether a private or
/// version-conditional API exists, and are written to tolerate getting NULL
/// back, exactly like a real device would return for a symbol that isn't
/// present. These are exempted from the "panic to flag a likely-missing
/// host function" behaviour below, since a panic here would be flagging a
/// false positive, not an actual gap.
const KNOWN_OPTIONAL_SYMBOLS: &[&str] = &[
    // Used by Mono/MonoTouch-based apps (e.g. Unity, MonoGame) to
    // conditionally call the (fairly obscure, version-gated) app icon
    // badge-number API.
    "__setBadgeNumber",
    // Same story, for probing whether Game Center is available (it was
    // introduced in iOS 4.1, so older-minimum-OS apps check for it before
    // using it).
    "__gameCenterIsGameCenterAvailable",
];

fn dlsym(env: &mut Environment, handle: MutVoidPtr, symbol: ConstPtr<u8>) -> MutVoidPtr {
    assert!(
        handle == RTLD_DEFAULT || is_known_library(env.mem.cstr_at_utf8(handle.cast()).unwrap())
    );
    // For some reason, the symbols passed to dlsym() don't have the leading _.
    let symbol = format!("_{}", env.mem.cstr_at_utf8(symbol).unwrap());
    // TODO: error handling. dlsym() should just return NULL in this case, but
    // currently it's probably more useful to have the emulator crash if there's
    // no symbol found, since it most likely indicates a missing host function.
    // TODO: Symbol lookup should be scoped to the specific library requested,
    // where appropriate!
    let addr = env
        .dyld
        .create_proc_address(&mut env.mem, &mut env.cpu, &symbol);
    match addr {
        Ok(addr) => Ptr::from_bits(addr.addr_with_thumb_bit()),
        Err(_) if KNOWN_OPTIONAL_SYMBOLS.contains(&symbol.as_str()) => Ptr::null(),
        Err(_) => panic!("dlsym() for unimplemented function {symbol}"),
    }
}

/// `int dladdr(const void *addr, Dl_info *info)`
///
/// Real `dladdr()` looks up which loaded image (and nearest exported symbol)
/// a given address belongs to. touchHLE's dyld only supports forward lookup
/// (name -> address), not reverse (address -> name), and building a full
/// reverse index just for this would be disproportionate. However, the
/// callers that use `dladdr()` in practice are crash reporters / self-
/// diagnostics (e.g. OpenFeint), which only need it to *not crash* and to
/// return a success/failure flag; the individual fields are best-effort.
///
/// Per the contract, on success we return non-zero and fill `info`. We set
/// the symbol fields to NULL (documented behaviour when no symbol can be
/// resolved for the address) and leave the image fields NULL as well, since
/// we don't track per-image base addresses here. This is a valid, non-
/// crashing response that satisfies the common "is this address in a known
/// image?" check without fabricating misleading data.
fn dladdr(env: &mut Environment, _addr: ConstVoidPtr, info: MutPtr<Dl_info>) -> i32 {
    if !info.is_null() {
        env.mem.write(
            info,
            Dl_info {
                dli_fname: Ptr::null(),
                dli_fbase: Ptr::null(),
                dli_sname: Ptr::null(),
                dli_saddr: Ptr::null(),
            },
        );
    }
    // Non-zero = success (note: dladdr's success convention is the opposite of
    // most libc functions).
    1
}

fn dlclose(env: &mut Environment, handle: MutVoidPtr) -> i32 {
    assert!(
        handle == RTLD_DEFAULT || is_known_library(env.mem.cstr_at_utf8(handle.cast()).unwrap())
    );
    0 // success
}

pub const FUNCTIONS: FunctionExports = &[
    export_c_func!(dlopen(_, _)),
    export_c_func!(dlsym(_, _)),
    export_c_func!(dladdr(_, _)),
    export_c_func!(dlclose(_)),
];
