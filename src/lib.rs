// Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0

//! This crate is a Rust version of CloudFront Client Routing Library. Functions
//! are provided to encode a label and prepend it to a domain and to decode a
//! label for verification purposes.

mod bitwise;
pub mod client_routing_label;
pub mod encode_decode;
pub mod errors;
pub mod hash;
pub mod ip;

use client_routing_label::{ClientRoutingLabel, DecodedClientRoutingLabel};
use errors::DecodeLengthError;
use hash::hash_cgid;
use ip::parse_client_ip;

/// Returns domain with client routing key prepended as a subdomain.
///
/// The encode function takes in 3 parameters: `client_ip`, `content_group_id`,
/// and `fqdn`. `client_ip` is parsed into
/// [`ClientSubnetEncodingData`](crate::ip::ClientSubnetEncodingData). `cgid` is
/// hashed into a 64 bit number via xxHash. That data is then encoded into a
/// client routing label and then returned prepended as a subdomain to the
/// `fqdn`.
///
/// # Examples:
/// ```
/// use amazon_cloudfront_client_routing_lib::encode_request_data;
/// 
/// // ipv4
/// let mut encoded_label = encode_request_data("1.2.3.4", "mv-456", "example.com");
/// assert_eq!("abacaqdaaaaaaaamnjg3oubcyvrgm.example.com", encoded_label);
///
/// // ipv6
/// encoded_label = encode_request_data("0102:0304:0506:0708:090a:0b0c:0d0e:0f10", "mv-456", "example.com");
/// assert_eq!("abqcaqdaqcqmaaaynjg3oubcyvrgm.example.com", encoded_label);
///
/// // invalid client_ip
/// encoded_label = encode_request_data("1.2.a", "mv-456", "example.com");
/// assert_eq!("abaaaaaaaaaaaaaanjg3oubcyvrgm.example.com", encoded_label);
///
/// // empty cgid
/// encoded_label = encode_request_data("1.2.3.4", "", "example.com");
/// assert_eq!("abacaqdaaaaaaaamaaaaaaaaaaaaa.example.com", encoded_label);
/// ```
pub fn encode_request_data(client_ip: &str, content_group_id: &str, fqdn: &str) -> String {
    let client_subnet_encoding_data = parse_client_ip(client_ip);

    let mut label = ClientRoutingLabel::default();

    label.set_data(client_subnet_encoding_data, hash_cgid(content_group_id));

    let client_routing_label = label.encode();
    format!("{}.{}", client_routing_label, fqdn)
}

/// Returns a result containing either a [`DecodedClientRoutingLabel`] or a
/// [`DecodeLengthError`].
///
/// The decode function takes in a &str param: `domain`. This domain can be a FQDN
/// or just the dns label generated by the [`encode_request_data`] function. It
/// decodes the string and formats it into a [`DecodedClientRoutingLabel`]. If the
/// client routing label is not the first DNS label or is not included in `domain`
/// a [`DecodeLengthError`] will be returned.
///
/// # Examples:
/// ```
/// use amazon_cloudfront_client_routing_lib::decode_request_data;
///
/// // valid client routing label
/// let decoded_label = decode_request_data("abacaqdaaaaaaaamnjg3oubcyvrgm");
/// match decoded_label {
///     Ok(data) => {
///         assert_eq!([1, 2, 3, 0, 0, 0, 0, 0], data.client_subnet);
///         assert_eq!(24, data.subnet_mask);
///         assert_eq!(false, data.is_ipv6);
///         assert_eq!(15319960192071419084, data.cgid);
///     },
///     Err(e) => panic!("Decoding error when there shouldn't be: {}", e)
/// };
/// 
/// // fqdn with valid client routing label
/// let decoded_label = decode_request_data("abacaqdaaaaaaaamnjg3oubcyvrgm.example.com");
/// match decoded_label {
///     Ok(data) => {
///         assert_eq!([1, 2, 3, 0, 0, 0, 0, 0], data.client_subnet);
///         assert_eq!(24, data.subnet_mask);
///         assert_eq!(false, data.is_ipv6);
///         assert_eq!(15319960192071419084, data.cgid);
///     },
///     Err(e) => panic!("Decoding error when there shouldn't be: {}", e)
/// };
/// 
/// // fqdn with subdomain and valid client routing label
/// let decoded_label = decode_request_data("abacaqdaaaaaaaamnjg3oubcyvrgm.vod1.example.com");
/// match decoded_label {
///     Ok(data) => {
///         assert_eq!([1, 2, 3, 0, 0, 0, 0, 0], data.client_subnet);
///         assert_eq!(24, data.subnet_mask);
///         assert_eq!(false, data.is_ipv6);
///         assert_eq!(15319960192071419084, data.cgid);
///     },
///     Err(e) => panic!("Decoding error when there shouldn't be: {}", e)
/// };
/// 
/// // fqdn without valid client routing label
/// let decoded_label = decode_request_data("example.com");
/// match decoded_label {
///     Ok(data) => panic!("Should have thrown a DecodeLengthError"),
///     Err(e) => {
///         assert_eq!(format!("{}", e), "Passed 7 - expected 29 characters");
///     }
/// };
/// 
/// // client routing label needs to be the first DNS label
/// let decoded_label = decode_request_data("vod1.abacaqdaaaaaaaamnjg3oubcyvrgm.example.com");
/// match decoded_label {
///     Ok(data) => panic!("Should have thrown a DecodeLengthError"),
///     Err(e) => {
///         assert_eq!(format!("{}", e), "Passed 4 - expected 29 characters");
///     }
/// };
///
/// // invalid
/// let decoded_label = decode_request_data("abacaqdaaaaaaaamnjg3oubcy"); // invalid length
/// match decoded_label {
///     Ok(data) => panic!("Should have thrown a DecodeLengthError"),
///     Err(e) => {
///         assert_eq!(format!("{}", e), "Passed 25 - expected 29 characters");
///     }
/// };
/// ```
pub fn decode_request_data(
    domain: &str,
) -> Result<DecodedClientRoutingLabel, DecodeLengthError> {
    let client_routing_label = domain.split(".").next().unwrap_or_default();
    let client_routing_label: &mut [u8] = &mut Box::from(client_routing_label.as_bytes());
    client_routing_label.make_ascii_lowercase();

    let mut label = ClientRoutingLabel::default();

    label.decode(client_routing_label)
}