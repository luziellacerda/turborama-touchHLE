/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */
//! CommonCrypto and friends

use crate::dyld::FunctionExports;
use crate::mem::{ConstVoidPtr, MutPtr, MutVoidPtr};
use crate::{export_c_func, Environment};
use digest::Digest;
use hmac::{Hmac, Mac};
use md5::Md5;
use sha1::Sha1;

// CCHmacAlgorithm values from <CommonCrypto/CommonHMAC.h>
const K_CC_HMAC_ALG_SHA1: u32 = 0;
const K_CC_HMAC_ALG_MD5: u32 = 1;

fn CC_MD5(env: &mut Environment, data: ConstVoidPtr, len: u32, md: MutPtr<u8>) -> MutPtr<u8> {
    let mut hasher = Md5::new();
    hasher.update(env.mem.bytes_at(data.cast(), len));
    let digest = hasher.finalize();
    env.mem.bytes_at_mut(md, 16).copy_from_slice(&digest[..]);
    md
}

fn CC_SHA1(env: &mut Environment, data: ConstVoidPtr, len: u32, md: MutPtr<u8>) -> MutPtr<u8> {
    let mut hasher = Sha1::new();
    hasher.update(env.mem.bytes_at(data.cast(), len));
    let digest = hasher.finalize();
    env.mem.bytes_at_mut(md, 20).copy_from_slice(&digest[..]);
    md
}

/// `void CCHmac(CCHmacAlgorithm algorithm, const void *key, size_t keyLength,
///              const void *data, size_t dataLength, void *macOut)`
///
/// Computes a keyed-hash MAC. `macOut` must point to a buffer large enough
/// for the chosen algorithm's digest (20 bytes for SHA1, 16 for MD5). Only
/// the algorithms whose underlying hash we already support are implemented;
/// others panic so a genuinely missing case is flagged rather than silently
/// producing a wrong MAC.
fn CCHmac(
    env: &mut Environment,
    algorithm: u32,
    key: ConstVoidPtr,
    key_length: u32,
    data: ConstVoidPtr,
    data_length: u32,
    mac_out: MutVoidPtr,
) {
    let key_bytes = env.mem.bytes_at(key.cast(), key_length).to_vec();
    let data_bytes = env.mem.bytes_at(data.cast(), data_length).to_vec();
    match algorithm {
        K_CC_HMAC_ALG_SHA1 => {
            let mut mac = Hmac::<Sha1>::new_from_slice(&key_bytes).unwrap();
            mac.update(&data_bytes);
            let result = mac.finalize().into_bytes();
            env.mem
                .bytes_at_mut(mac_out.cast(), 20)
                .copy_from_slice(&result[..]);
        }
        K_CC_HMAC_ALG_MD5 => {
            let mut mac = Hmac::<Md5>::new_from_slice(&key_bytes).unwrap();
            mac.update(&data_bytes);
            let result = mac.finalize().into_bytes();
            env.mem
                .bytes_at_mut(mac_out.cast(), 16)
                .copy_from_slice(&result[..]);
        }
        _ => unimplemented!("CCHmac algorithm {}", algorithm),
    }
}

pub const FUNCTIONS: FunctionExports = &[
    export_c_func!(CC_MD5(_, _, _)),
    export_c_func!(CC_SHA1(_, _, _)),
    export_c_func!(CCHmac(_, _, _, _, _, _)),
];
