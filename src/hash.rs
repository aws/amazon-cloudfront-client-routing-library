// Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0

use std::hash::Hasher;
use twox_hash::XxHash64;

/// Utilizes xxHash to hash a `cgid` into a 64 bit number and returns that
/// number.
///
/// Passing an empty string as the `cgid` will result in 0 being returned
/// instead of the hash of `cgid`.
///
/// # Examples
/// ```
/// use amazon_cloudfront_client_routing_lib::hash::hash_cgid;
///
/// // valid cgid
/// let mut hashed_cgid = hash_cgid("f3663718-7699-4e6e-b482-daa2f690cf64");
/// assert_eq!(8517775255794402596, hashed_cgid);
///
/// // empty cgid
/// hashed_cgid = hash_cgid("");
/// assert_eq!(0, hashed_cgid);
/// ```
pub fn hash_cgid(cgid: &str) -> u64 {
    if cgid.is_empty() {
        return 0;
    }

    let mut hasher = XxHash64::default();
    hasher.write(cgid.as_bytes());

    hasher.finish()
}

#[cfg(test)]
mod tests {
    use super::hash_cgid;

    #[test]
    fn validate_hash_cgid() {
        assert_eq!(9402033733208250942, hash_cgid("SM89P"));
        assert_eq!(16745045142164894816, hash_cgid("DP0124QHYT"));
        assert_eq!(15007018045908736946, hash_cgid("b086vx9VmK"));
        assert_eq!(15151312625956013430, hash_cgid("abcdefghijhjuio"));
        assert_eq!(
            8696017447135811798,
            hash_cgid("VZ9C5G6H12PC5GH7Y0ABCDEFGHIJHJUIOZZAA1")
        );
    }

    #[test]
    fn validate_hash_similar_cgids_not_equal() {
        assert_ne!(hash_cgid("SM89P"), hash_cgid("sm89p"));
        assert_ne!(hash_cgid("abcdefghijhjuio0"), hash_cgid("abcdefghijhjuio"));
        assert_ne!(hash_cgid("B086VX9VMK "), hash_cgid("B086VX9VMK"));
        assert_ne!(
            hash_cgid("hfquwah9tds\u{00}"),
            hash_cgid("hfquwah9tds\u{01}")
        );
    }

    #[test]
    fn validate_hash_empty_cgid_zero() {
        assert_eq!(0, hash_cgid(""));
    }
}
