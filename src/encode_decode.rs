// Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0

use crate::{client_routing_label::EncodableData, errors::DecodeLengthError, bitwise::get_mask};

const BASE32_ALPHABET: &[u8] = b"abcdefghijklmnopqrstuvwxyz234567";
const BASE32_NUM_BITS_IN_CHAR: u8 = 5;
const MAX_DNS_LABEL_SIZE: u8 = 63;

/// Struct for encoding, decoding, and validating [`EncodableData`] with Base32.
/// 
/// Uses lowercase version of the RFC 4648 Base32 alphabet. Methods treat each
/// set of 5 bits in [`EncodableData`] as a separate character. Invalid characters
/// will be treated as 'a' instead of marking the entire label as invalid for
/// efficiency. Contains no properties, for usage see individual functions or
/// [`ClientRoutingLabel`](crate::client_routing_label::ClientRoutingLabel).
#[derive(Copy, Clone, Debug)]
pub struct Base32 {}

impl Base32 {
    /// Returns a lowercase Base32 string encoded from `encodable_data`.
    /// 
    /// Iterates over `encodable_data`, encoding bits from `value` until 
    /// not enough bits remain to make a full char. Remaining bits are
    /// then used in the subsequent iteration. After iterating over
    /// everything, if there are not enough bits to make a char 0 will
    /// be used to pad the left over bits. Encoding uses a lowercase
    /// version of the RFC 4648 Base32 alphabet.
    /// 
    /// # Examples:
    /// ```
    /// use amazon_cloudfront_client_routing_lib::encode_decode::Base32;
    /// use amazon_cloudfront_client_routing_lib::client_routing_label::EncodableData;
    /// 
    /// let encoding_system = Base32 {};
    /// let encodable_data = &mut [
    ///     EncodableData { // 0b01010 => "k"
    ///         value: 10,
    ///         num_bits: 5
    ///     },
    ///     EncodableData { // 0b00011_11011 => "d3"
    ///         value: 123,
    ///         num_bits: 10
    ///     },
    ///     EncodableData { // 0b0 => "a"
    ///         value: 0,
    ///         num_bits: 1
    ///     },
    /// ];
    /// 
    /// assert_eq!("kd3a", encoding_system.encode(encodable_data));
    /// ```
    pub fn encode(&self, encodable_data: &mut [EncodableData]) -> String {
        let value_mask: u64 = get_mask(BASE32_NUM_BITS_IN_CHAR);
        let mut encoded_data: Vec<char> = Vec::with_capacity(MAX_DNS_LABEL_SIZE as usize);
        let mut value_to_encode: u8 = 0;
        let mut num_bits_left_over: u8 = 0;
        for data in encodable_data.iter_mut() {
            while data.has_bits_for_char(BASE32_NUM_BITS_IN_CHAR - num_bits_left_over) {
                value_to_encode += data.get_next_bits_to_encode(BASE32_NUM_BITS_IN_CHAR - num_bits_left_over);
                encoded_data.push(BASE32_ALPHABET[value_to_encode as usize] as char);

                num_bits_left_over = 0;
                value_to_encode = 0;
            }

            value_to_encode |= ((data.value << (BASE32_NUM_BITS_IN_CHAR - (data.num_bits + num_bits_left_over))) & value_mask) as u8;
            num_bits_left_over += data.num_bits;
        }

        if num_bits_left_over > 0 {
            encoded_data.push(BASE32_ALPHABET[value_to_encode as usize] as char);
        }

        encoded_data.iter().collect()
    }

    /// Validates `client_routing_label` is the proper length to fit `total_num_bits`.
    /// 
    /// Calculates how many chars would be encoded for `total_num_bits` and then
    /// checks if the `client_routing_label` has that many chars. Returns a [`Result`]
    /// with '()' if it's valid or a [`DecodeLengthError`] if it's not valid.
    /// 
    /// # Examples:
    /// ```
    /// use amazon_cloudfront_client_routing_lib::encode_decode::Base32;
    /// 
    /// let encoding_system = Base32 {};
    /// 
    /// // valid
    /// match encoding_system.is_valid_client_routing_label(145, b"abaaaaaaaaaaaaaaaaaaaackvj5oa") {
    ///     Ok(()) => (),
    ///     Err(_e) => panic!("Threw error when shouldn't have.")
    /// };
    /// 
    /// // invalid
    /// match encoding_system.is_valid_client_routing_label(145, b"abaaaaaaaaaaaaaaaaaaaackvj5oabcd") {
    ///     Ok(()) => (),
    ///     Err(e) => assert_eq!("Passed 32 - expected 29 characters", e.to_string())
    /// };
    /// ```
    pub fn is_valid_client_routing_label(
        &self,
        total_num_bits: u8,
        client_routing_label: &[u8],
    ) -> Result<(), DecodeLengthError> {
        if client_routing_label.len() as u8
            != (total_num_bits + BASE32_NUM_BITS_IN_CHAR - 1) / BASE32_NUM_BITS_IN_CHAR
        {
            let e = DecodeLengthError {
                num_chars: client_routing_label.len(),
                expected_num_chars: ((total_num_bits + BASE32_NUM_BITS_IN_CHAR - 1)
                    / BASE32_NUM_BITS_IN_CHAR) as usize,
            };
            return Err(e);
        }

        Ok(())
    }

    /// Sets `encodable_data` based on passed `encoded_label`.
    /// 
    /// Validates `encoded_label` is valid based on `total_num_bits`. If not valid,
    /// returns a [`Result`] containing [`DecodeLengthError`]. If valid, iterates
    /// over `encodable_data` and sets each value based on the label value. Invalid
    /// characters in a label are treated as if they had a value of 0.
    /// 
    /// # Examples:
    /// ```
    /// use amazon_cloudfront_client_routing_lib::encode_decode::Base32;
    /// use amazon_cloudfront_client_routing_lib::client_routing_label::EncodableData;
    /// 
    /// let encoding_system = Base32 {};
    /// 
    /// // valid
    /// let encodable_data = &mut [
    ///     EncodableData {
    ///         value: 0,
    ///         num_bits: 5
    ///     },
    ///     EncodableData {
    ///         value: 0,
    ///         num_bits: 10
    ///     },
    ///     EncodableData {
    ///         value: 0,
    ///         num_bits: 1
    ///     },
    /// ];
    /// 
    /// match encoding_system.decode(encodable_data, b"kd3a", 16) {
    ///     Ok(()) => {
    ///         assert_eq!(10, encodable_data[0].value);
    ///         assert_eq!(123, encodable_data[1].value);
    ///         assert_eq!(0, encodable_data[2].value);
    ///     },
    ///     Err(_e) => panic!("Threw error when shouldn't have.")
    /// };
    /// 
    /// // invalid
    /// match encoding_system.decode(encodable_data, b"kd3a", 10) {
    ///     Ok(()) => panic!("Didn't throw error when should have."),
    ///     Err(e) => assert_eq!("Passed 4 - expected 2 characters", e.to_string())
    /// };
    /// ```
    pub fn decode(
        &self,
        encodable_data: &mut [EncodableData],
        encoded_label: &[u8],
        total_num_bits: u8,
    ) -> Result<(), DecodeLengthError> {
        match self.is_valid_client_routing_label(total_num_bits, encoded_label) {
            Ok(()) => (),
            Err(e) => return Err(e),
        };

        let mut label_values: Vec<u8> = encoded_label
            .iter()
            .map(|a| BASE32_ALPHABET.iter().position(|b| a == b).unwrap_or(0) as u8)
            .collect();

        let mut num_bits_in_char: u8 = BASE32_NUM_BITS_IN_CHAR;
        let mut label_index: usize = 0;
        for data in encodable_data.iter_mut() {
            let original_num_bits: u8 = data.num_bits;
            data.value = 0;
            
            while data.has_bits_for_char(num_bits_in_char) {
                data.add_bits(num_bits_in_char, label_values[label_index]);
                label_index += 1;
                num_bits_in_char = BASE32_NUM_BITS_IN_CHAR;
            }
            
            if data.num_bits > 0 {
                num_bits_in_char -= data.num_bits;
                data.add_bits(data.num_bits, label_values[label_index] >> num_bits_in_char);
                label_values[label_index] &= get_mask(num_bits_in_char) as u8;
            }

            data.num_bits = original_num_bits;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::client_routing_label::EncodableData;
    
    // All the data has values with bit size <= num_bits.
    // Total bits is divisible by 5 and can be encoded with no padding.
    #[test]
    fn validate_encode_value_proper_size_no_padding_needed() {
        let encoding_system = Base32 {};
        let encodable_data = &mut [
            EncodableData {
                value: 0,
                num_bits: 109,
            },
            EncodableData {
                value: 1,
                num_bits: 5,
            },
            EncodableData {
                value: 0,
                num_bits: 1,
            },
            EncodableData {
                value: 6148494311290830848,
                num_bits: 64,
            },
            EncodableData {
                value: 24,
                num_bits: 6,
            },
            EncodableData {
                value: 957415,
                num_bits: 20,
            },
        ];

        assert_eq!("aaaaaaaaaaaaaaaaaaaaaackvj5oaaaaaaaay5g7h", encoding_system.encode(encodable_data));
    }

    // Some data has a value with bit size > num_bits.
    // Total bits is divisible by 5 and can be encoded with no padding.
    #[test]
    fn validate_encode_value_proper_size_padding_needed() {
        let encoding_system = Base32 {};
        let encodable_data = &mut [
            EncodableData {
                value: 36,
                num_bits: 12,
            },
            EncodableData {
                value: 3734643,
                num_bits: 22,
            },
            EncodableData {
                value: 2367,
                num_bits: 14,
            },
        ];

        assert_eq!("ajhd6hgjh4", encoding_system.encode(encodable_data));
    }

    // All the data has values with bit size <= num_bits.
    // Total bits is not divisible by 5 and will need padding to encode.
    #[test]
    fn validate_encode_value_too_large_no_padding_needed() {
        let encoding_system = Base32 {};
        let encodable_data: &mut [EncodableData] = &mut [
            EncodableData {
                value: 5346,
                num_bits: 5,
            },
            EncodableData {
                value: 3474,
                num_bits: 56,
            },
            EncodableData {
                value: 0,
                num_bits: 14,
            },
            EncodableData {
                value: 0,
                num_bits: 8,
            },
            EncodableData {
                value: 46374,
                num_bits: 83,
            },
        ];

        assert_eq!("caaaaaaaabwjaaaaaaaaaaaaaaaaaawuta", encoding_system.encode(encodable_data));
    }

    // Some data has a value with bit size > num_bits.
    // Total bits is not divisible by 5 and will need padding to encode.
    #[test]
    fn validate_encode_value_too_large_padding_needed() {
        let encoding_system = Base32 {};
        let encodable_data: &mut [EncodableData] = &mut [
            EncodableData {
                value: 2423,
                num_bits: 5,
            },
            EncodableData {
                value: 432,
                num_bits: 3,
            },
            EncodableData {
                value: 31,
                num_bits: 12,
            },
            EncodableData {
                value: 43,
                num_bits: 10,
            },
            EncodableData {
                value: 64,
                num_bits: 6,
            },
        ];

        assert_eq!("xaa7blaa", encoding_system.encode(encodable_data));
    }

    #[test]
    fn validate_encode_empty_data() {
        let encoding_system = Base32 {};
        let encodable_data: &mut [EncodableData] = &mut [];

        assert_eq!("", encoding_system.encode(encodable_data));
    }

    #[test]
    fn validate_encode_not_enough_data_for_char() {
        let encoding_system = Base32 {};
        let encodable_data: &mut [EncodableData] = &mut [
            EncodableData {
                value: 1,
                num_bits: 1,
            },
            EncodableData {
                value: 2,
                num_bits: 2,
            },
        ];

        assert_eq!("y", encoding_system.encode(encodable_data));
    }

    #[test]
    fn validate_decode_label_with_no_padding() {
        let encoding_system = Base32 {};
        let encodable_data = &mut [
            EncodableData {
                value: 0,
                num_bits: 109,
            },
            EncodableData {
                value: 0,
                num_bits: 5,
            },
            EncodableData {
                value: 0,
                num_bits: 1,
            },
            EncodableData {
                value: 0,
                num_bits: 64,
            },
            EncodableData {
                value: 0,
                num_bits: 6,
            },
            EncodableData {
                value: 0,
                num_bits: 20,
            },
        ];
        
        match encoding_system.decode(encodable_data, b"aaaaaaaaaaaaaaaaaaaaaackvj5oaaaaaaaay5g7h", 205) {
            Ok(()) => {
                assert_eq!(0, encodable_data[0].value);
                assert_eq!(1, encodable_data[1].value);
                assert_eq!(0, encodable_data[2].value);
                assert_eq!(6148494311290830848, encodable_data[3].value);
                assert_eq!(24, encodable_data[4].value);
                assert_eq!(957415, encodable_data[5].value);
            },
            Err(e) => panic!("Threw error when shouldn't have: {}", e.to_string())
        };
    }

    #[test]
    fn validate_decode_label_with_padding() {
        let encoding_system = Base32 {};
        let encodable_data = &mut [
            EncodableData {
                value: 0,
                num_bits: 12,
            },
            EncodableData {
                value: 0,
                num_bits: 22,
            },
            EncodableData {
                value: 0,
                num_bits: 14,
            },
        ];
        
        match encoding_system.decode(encodable_data, b"ajhd6hgjh4", 48) {
            Ok(()) => {
                assert_eq!(36, encodable_data[0].value);
                assert_eq!(3734643, encodable_data[1].value);
                assert_eq!(2367, encodable_data[2].value);
            },
            Err(e) => panic!("Threw error when shouldn't have: {}", e.to_string())
        };
    }

    #[test]
    fn validate_decode_data_already_has_value() {
        let encoding_system = Base32 {};
        let encodable_data = &mut [
            EncodableData {
                value: 2423,
                num_bits: 5,
            },
            EncodableData {
                value: 53,
                num_bits: 3,
            },
            EncodableData {
                value: 43,
                num_bits: 12,
            },
            EncodableData {
                value: 754,
                num_bits: 10,
            },
            EncodableData {
                value: 34,
                num_bits: 6,
            },
        ];
        
        match encoding_system.decode(encodable_data, b"xaa7blaa", 36) {
            Ok(()) => {
                assert_eq!(23, encodable_data[0].value);
                assert_eq!(0, encodable_data[1].value);
                assert_eq!(31, encodable_data[2].value);
                assert_eq!(43, encodable_data[3].value);
                assert_eq!(0, encodable_data[4].value);
            },
            Err(e) => panic!("Threw error when shouldn't have: {}", e.to_string())
        };
    }

    #[test]
    fn validate_decode_empty_label() {
        let encoding_system = Base32 {};
        let encodable_data = &mut [];
        
        match encoding_system.decode(encodable_data, b"", 0) {
            Ok(()) => {},
            Err(e) => panic!("Threw error when shouldn't have: {}", e.to_string())
        };
    }

    #[test]
    fn validate_decode_label_too_large() {
        let encoding_system = Base32 {};
        let encodable_data = &mut [
            EncodableData {
                value: 0,
                num_bits: 12,
            },
            EncodableData {
                value: 0,
                num_bits: 22,
            },
            EncodableData {
                value: 0,
                num_bits: 14,
            },
        ];
        
        match encoding_system.decode(encodable_data, b"abacabacdfed", 46) {
            Ok(()) => panic!("Didn't throw error when should have"),
            Err(e) => assert_eq!("Passed 12 - expected 10 characters", e.to_string())
        };
    }

    #[test]
    fn validate_decode_label_too_small() {
        let encoding_system = Base32 {};
        let encodable_data = &mut [
            EncodableData {
                value: 0,
                num_bits: 12,
            },
            EncodableData {
                value: 0,
                num_bits: 22,
            },
            EncodableData {
                value: 0,
                num_bits: 14,
            },
        ];
        
        match encoding_system.decode(encodable_data, b"aba", 46) {
            Ok(()) => panic!("Didn't throw error when should have"),
            Err(e) => assert_eq!("Passed 3 - expected 10 characters", e.to_string())
        };
    }
}
