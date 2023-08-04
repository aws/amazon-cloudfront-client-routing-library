// Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0

use crate::bitwise::get_mask;
use crate::encode_decode::Base32;
use crate::errors::DecodeLengthError;
use crate::ip::ClientSubnetEncodingData;

const CLIENT_ROUTING_LABEL_VERSION: u16 = 1;

/// Struct containing decoded client routing label values.
///
/// Consist of 5 properties: `client_sdk_version`, `is_ipv6`, `client_subnet`,
/// `subnet_mask`, and `cgid`. Each property maps directly to a value in
/// [`ClientRoutingLabel`], with the minimal type needed to represent that data.
/// Only the least significant bits will be kept in the case `value` can't fit in
/// `num_bits`.
///
/// # Examples:
/// ```
/// use amazon_cloudfront_client_routing_lib::client_routing_label::DecodedClientRoutingLabel;
///
/// let client_sdk_version: u16 = 1;
/// let is_ipv6: bool = false;
/// let client_subnet: [u8; 8] = [1, 2, 3, 0, 0, 0, 0, 0];
/// let subnet_mask: u8 = 24;
/// let cgid: u64 = 15151312625956013430;
///
/// let decoded_client_routing_label = DecodedClientRoutingLabel {
///     client_sdk_version,
///     is_ipv6,
///     client_subnet,
///     subnet_mask,
///     cgid
/// };
/// ```
#[derive(Copy, Clone, Debug)]
pub struct DecodedClientRoutingLabel {
    pub client_sdk_version: u16,
    pub is_ipv6: bool,
    pub client_subnet: [u8; 8],
    pub subnet_mask: u8,
    pub cgid: u64,
}

/// Struct containing data to encode in a [`ClientRoutingLabel`].
///
/// Consist of 2 properties: `value`, and `num_bits`. `value` is a u64 and
/// should be set to the actual data to encode. `num_bits` is a u8 and should be
/// set to how many bits should be encoded. This ensures a particular value will
/// always be encoded to the same bit position in a label, regardless of the
/// actual size of value.
///
/// # Examples:
/// ```
/// use amazon_cloudfront_client_routing_lib::client_routing_label::EncodableData;
/// use amazon_cloudfront_client_routing_lib::encode_decode::Base32;
///
/// let mut data: EncodableData;
/// let encoding_system = Base32 {};
///
/// // value is 1 bit and needs to encode as 10 bits: 0b0000000001
/// data = EncodableData {
///     value: 1,
///     num_bits: 10,
/// };
///
/// assert_eq!("ab", encoding_system.encode(&mut [data]));
///
/// // value is 4 bits and needs to encode as 5 bits: 0b10000
/// data = EncodableData {
///     value: 16,
///     num_bits: 5
/// };
///
/// assert_eq!("q", encoding_system.encode(&mut [data]));
///
/// // value is 6 bits and needs to encode as 5 bits: 0b00000
/// // only the least significant bits are retained
/// data = EncodableData {
///     value: 32,
///     num_bits: 5
/// };
///
/// assert_eq!("a", encoding_system.encode(&mut [data]));
#[derive(Copy, Clone, Debug)]
pub struct EncodableData {
    pub value: u64,
    pub num_bits: u8,
}

impl EncodableData {
    /// Returns `num_bits_needed` from the front of [`EncodableData`].
    /// 
    /// Masks and shifts `value` so the bits in the proper location are returned.
    /// If [`EncodableData`] has a larger `num_bits` than bits in the actual `value`,
    /// 0 will be returned. Decreases `num_bits` by `num_bits_needed` to keep track
    /// of how many bits are left to encode.
    /// 
    /// `num_bits_needed` needs to be an integer 1-8 because the max bit size for a
    /// character for any encoding system up to base 256 is 8 bits. This function will
    /// also throw an error if `num_bits_needed` is bigger than `num_bits`.
    /// 
    /// # Examples:
    /// ```
    /// use amazon_cloudfront_client_routing_lib::client_routing_label::EncodableData;
    /// 
    /// let mut encodable_data = EncodableData {
    ///     value: 10, // value can be represented by 4 bits: 0b1010
    ///     num_bits: 6 // specifying 6 bits means it should be encoded as: 0b001010
    /// };
    /// 
    /// assert_eq!(2, encodable_data.get_next_bits_to_encode(4)); // 0b0010
    /// assert_eq!(2, encodable_data.get_next_bits_to_encode(2)); // 0b10
    /// ```
    pub fn get_next_bits_to_encode(&mut self, num_bits_needed: u8) -> u8 {
        self.num_bits -= num_bits_needed;
        let mask: u128 = (get_mask(num_bits_needed) as u128) << self.num_bits;
        let bits_to_encode = (self.value as u128 & mask) >> self.num_bits;

        self.value &= get_mask(self.num_bits);
        
        bits_to_encode as u8
    }

    /// Determines if there are enough bits in `num_bits` to make a char.
    /// 
    /// Takes one parameter: `num_bits_in_char`. `num_bits_in_char` should
    /// be determined by the encoding system e.g. 5 bits for a char in base32 encoding.
    /// 
    /// # Examples:
    /// ```
    /// use amazon_cloudfront_client_routing_lib::client_routing_label::EncodableData;
    /// 
    /// let encodable_data = EncodableData {
    ///     value: 10,
    ///     num_bits: 6
    /// };
    /// 
    /// assert_eq!(true, encodable_data.has_bits_for_char(5));
    /// ```
    pub fn has_bits_for_char(self, num_bits_in_char: u8) -> bool {
        self.num_bits >= num_bits_in_char
    }

    /// Adds `value_to_add` to `value` and decrements `num_bits` by `num_bits_to_add`.
    /// 
    /// Intended to be used when decoding a value. `value` will be left shifted by
    /// `num_bits_to_add` and then `value_to_add` gets shifted in. This ensures
    /// bits can be added in their proper places. `num_bits` gets decremented to
    /// keep track of how many bits are still needed to fill [`EncodableData`].
    /// 
    /// # Examples:
    /// ```
    /// use amazon_cloudfront_client_routing_lib::client_routing_label::EncodableData;
    /// 
    /// let mut encodable_data = EncodableData {
    ///     value: 0,
    ///     num_bits: 10
    /// };
    /// 
    /// encodable_data.add_bits(6, 21);
    /// assert_eq!(21, encodable_data.value);
    /// 
    /// encodable_data.add_bits(3, 6);
    /// assert_eq!(174, encodable_data.value);
    /// ```
    pub fn add_bits(&mut self, num_bits_to_add: u8, value_to_add: u8) {
        self.num_bits -= num_bits_to_add;
        self.value <<= num_bits_to_add;
        self.value |= value_to_add as u64;
    }
}

/// Struct containing data to encode and what encoding system to use.
///
/// Consist of 2 properties: `encodable_data` and `encoding_system`.
/// `encodable_data` should be an array of 5 EncodableData items. The Default
/// implementation should be used for creating this struct to ensure each item
/// in the `encodable_data` contains the proper `num_bits` value.
///
/// # Examples
/// ```
/// use amazon_cloudfront_client_routing_lib::client_routing_label::ClientRoutingLabel;
///
/// let mut client_routing_label = ClientRoutingLabel::default();
/// client_routing_label.encodable_data[0].value = 1; // sdk version
/// client_routing_label.encodable_data[1].value = 1; // is ipv6
/// client_routing_label.encodable_data[2].value = 9340004030419828736; // client subnet
/// client_routing_label.encodable_data[3].value = 48; // subnet mask
/// client_routing_label.encodable_data[4].value = 8517775255794402596; // cgid
/// ```
#[derive(Copy, Clone, Debug)]
pub struct ClientRoutingLabel {
    pub encodable_data: [EncodableData; 5],
    pub encoding_system: Base32,
}

impl Default for ClientRoutingLabel {
    fn default() -> Self {
        let sdk_version = EncodableData {
            value: CLIENT_ROUTING_LABEL_VERSION as u64,
            num_bits: 10,
        };
        let is_ipv6: EncodableData = EncodableData {
            value: 0,
            num_bits: 1,
        };
        let client_subnet = EncodableData {
            value: 0,
            num_bits: 64,
        };
        let subnet_mask = EncodableData {
            value: 0,
            num_bits: 6,
        };
        let cgid = EncodableData {
            value: 0,
            num_bits: 64,
        };
        Self {
            encodable_data: [sdk_version, is_ipv6, client_subnet, subnet_mask, cgid],
            encoding_system: Base32 {},
        }
    }
}

impl ClientRoutingLabel {
    /// Sets client subnet and cgid data in [`ClientRoutingLabel`].
    ///
    /// Takes in 2 parameters: `client_subnet_encoding_data` and `cgid`.
    /// `client_subnet_encoding_data` should be a [`ClientSubnetEncodingData`]
    /// struct and has the formatted values for `is_ipv6`, `client_subnet`, and
    /// `subnet_mask`.
    ///
    /// # Examples:
    /// ```
    /// use amazon_cloudfront_client_routing_lib::client_routing_label::ClientRoutingLabel;
    /// use amazon_cloudfront_client_routing_lib::ip::ClientSubnetEncodingData;
    ///
    /// let cgid = 8517775255794402596;
    /// let client_subnet_encoding_data = ClientSubnetEncodingData {
    ///     is_ipv6: 0,
    ///     client_subnet: 6148494311290830848,
    ///     subnet_mask: 24,
    /// };
    ///
    /// let mut client_routing_label = ClientRoutingLabel::default();
    /// client_routing_label.set_data(client_subnet_encoding_data, cgid);
    /// ```
    pub fn set_data(&mut self, client_subnet_encoding_data: ClientSubnetEncodingData, cgid: u64) {
        self.encodable_data[1].value = client_subnet_encoding_data.is_ipv6;
        self.encodable_data[2].value = client_subnet_encoding_data.client_subnet;
        self.encodable_data[3].value = client_subnet_encoding_data.subnet_mask;
        self.encodable_data[4].value = cgid;
    }

    /// Encodes `encodable_data` and returns encoded client routing label
    ///
    /// Calls the encode function of `encoding_system`. Each [`EncodableData`]
    /// item in `encodable_data` is formatted to the proper number of bits and
    /// encoded into a string.
    ///
    /// # Examples:
    /// ```
    /// use amazon_cloudfront_client_routing_lib::client_routing_label::ClientRoutingLabel;
    /// use amazon_cloudfront_client_routing_lib::ip::ClientSubnetEncodingData;
    ///
    /// let cgid = 8517775255794402596;
    /// let client_subnet_encoding_data = ClientSubnetEncodingData {
    ///     is_ipv6: 0,
    ///     client_subnet: 6148494311290830848,
    ///     subnet_mask: 24,
    /// };
    ///
    /// let mut client_routing_label = ClientRoutingLabel::default();
    /// client_routing_label.set_data(client_subnet_encoding_data, cgid);
    ///
    /// assert_eq!("abfku6xaaaaaaaamhmnjxo5hdzrje", client_routing_label.encode());
    /// ```
    pub fn encode(&mut self) -> String {
        self.encoding_system.encode(&mut self.encodable_data)
    }

    /// Decodes `client_routing_label` and returns a result containing either a
    /// [`DecodedClientRoutingLabel`] or a [`DecodeLengthError`] if the
    /// `client_routing_label` is invalid.
    ///
    /// # Examples:
    /// ```
    /// use amazon_cloudfront_client_routing_lib::client_routing_label::ClientRoutingLabel;
    ///
    /// let mut client_routing_label = ClientRoutingLabel::default();
    ///
    /// let decode_result = client_routing_label.decode(b"abfku6xaaaaaaaamhmnjxo5hdzrje");
    ///
    /// match decode_result {
    ///     Ok(decoded_client_routing_label) => {
    ///         assert_eq!(1, decoded_client_routing_label.client_sdk_version);
    ///         assert_eq!(false, decoded_client_routing_label.is_ipv6);
    ///         assert_eq!([85, 83, 215, 0, 0, 0, 0, 0], decoded_client_routing_label.client_subnet);
    ///         assert_eq!(24, decoded_client_routing_label.subnet_mask);
    ///         assert_eq!(8517775255794402596, decoded_client_routing_label.cgid);
    ///     },
    ///     Err(_e) => panic!("Decoding experienced an error when it shouldn't have")
    /// };
    /// ```
    pub fn decode(
        &mut self,
        client_routing_label: &[u8],
    ) -> Result<DecodedClientRoutingLabel, DecodeLengthError> {
        let total_num_bits = self.get_total_num_bits();
        let decoded_label = self.encoding_system.decode(
            &mut self.encodable_data,
            client_routing_label,
            total_num_bits,
        );

        match decoded_label {
            Ok(_value) => Ok(self.get_decoded_client_routing_label()),
            Err(e) => Err(e),
        }
    }

    /// Returns total num bits a label contains.
    ///
    /// Iterates over each item in `encodable_data` and sums the `num_bits` for
    /// each item, then returns that sum.
    ///
    /// # Examples:
    /// ```
    /// use amazon_cloudfront_client_routing_lib::client_routing_label::ClientRoutingLabel;
    ///
    /// let mut client_routing_label = ClientRoutingLabel::default();
    /// assert_eq!(145, client_routing_label.get_total_num_bits());
    /// ```
    pub fn get_total_num_bits(&mut self) -> u8 {
        self.encodable_data.iter().fold(0, |a, b| a + b.num_bits)
    }

    /// Creates and returns [`DecodedClientRoutingLabel`] based on
    /// `encodable_data`.
    fn get_decoded_client_routing_label(&mut self) -> DecodedClientRoutingLabel {
        DecodedClientRoutingLabel {
            client_sdk_version: self.encodable_data[0].value as u16,
            is_ipv6: self.encodable_data[1].value != 0,
            client_subnet: self.encodable_data[2].value.to_be_bytes(),
            subnet_mask: self.encodable_data[3].value as u8,
            cgid: self.encodable_data[4].value,
        }
    }
}
