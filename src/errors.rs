// Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0

use std::fmt;

/// Error struct used when decoding a client routing label key of an improper
/// length.
///
/// # Examples:
/// ```
/// use amazon_cloudfront_client_routing_lib::errors::DecodeLengthError;
///
/// let error = DecodeLengthError {
///     num_chars: 10,
///     expected_num_chars: 29,
/// };
///
/// assert_eq!("Passed 10 - expected 29 characters", error.to_string());
/// ```
#[derive(Debug, Copy, Clone)]
pub struct DecodeLengthError {
    pub num_chars: usize,
    pub expected_num_chars: usize,
}

impl std::error::Error for DecodeLengthError {}

impl fmt::Display for DecodeLengthError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "Passed {} - expected {} characters",
            self.num_chars, self.expected_num_chars,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::DecodeLengthError;

    #[test]
    fn validate_decode_length_error_text() {
        let error = DecodeLengthError {
            num_chars: 10,
            expected_num_chars: 29,
        };

        assert_eq!(error.to_string(), "Passed 10 - expected 29 characters");
    }
}
