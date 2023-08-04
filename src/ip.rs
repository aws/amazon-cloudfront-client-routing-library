// Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0

use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};

enum SubnetMask {
    Ipv4 = 24,
    Ipv6 = 48,
}

/// Struct containing 3 values needed for encoding: `client_subnet`,
/// `subnet_mask`, and `is_ipv6`.
///
/// Each property is a u64 because
/// [`EncodableData`](crate::client_routing_label::EncodableData) expects a u64.
/// `client_subnet` should be formatted as a u64 by shifting each octet into an
/// unsigned integer, applying the subnet mask, then shifting until the number
/// conforms to 64 bits.
///
/// # Examples
/// ```
/// use amazon_cloudfront_client_routing_lib::ip::ClientSubnetEncodingData;
///
/// // raw values
/// let client_ip = [1, 2, 3, 4];
/// let subnet_mask = 24;
/// let is_ipv6 = 0;
///
/// // shift bytes into int
/// let mut client_subnet = u32::from_be_bytes(client_ip) as u64;
/// // apply subnet mask
/// client_subnet &= 0xffffff00;
/// // conform to 64 bits
/// client_subnet <<= 32;
///
/// let client_subnet_encoding_data = ClientSubnetEncodingData {
///     client_subnet,
///     subnet_mask,
///     is_ipv6,
/// };
/// ```
pub struct ClientSubnetEncodingData {
    pub client_subnet: u64,
    pub subnet_mask: u64,
    pub is_ipv6: u64,
}

/// Parses passed `client_ip` into various data, returns
/// [`ClientSubnetEncodingData`].
///
/// Takes in one param: `client_ip`. Attempts to parse the `client_ip` into an
/// [`IpAddr`]. If successful, determines if it's an [`Ipv4Addr`] or an
/// [`Ipv6Addr`]. Returns [`ClientSubnetEncodingData`] with the parsed
/// information. If unsuccessful, returns [`ClientSubnetEncodingData`] with all
/// properties set to 0.
///
/// # Examples:
/// ```
/// use amazon_cloudfront_client_routing_lib::ip::parse_client_ip;
///
/// // Ipv4
/// let mut client_subnet_encoding_data = parse_client_ip("1.2.3.4");
/// assert_eq!([1, 2, 3, 0, 0, 0, 0, 0], client_subnet_encoding_data.client_subnet.to_be_bytes());
/// assert_eq!(24, client_subnet_encoding_data.subnet_mask);
/// assert_eq!(0, client_subnet_encoding_data.is_ipv6);
///
/// // Ipv6
/// client_subnet_encoding_data = parse_client_ip("0102:0304:0506:0708:090a:0b0c:0d0e:0f10");
/// assert_eq!([1, 2, 3, 4, 5, 6, 0, 0], client_subnet_encoding_data.client_subnet.to_be_bytes());
/// assert_eq!(48, client_subnet_encoding_data.subnet_mask);
/// assert_eq!(1, client_subnet_encoding_data.is_ipv6);
///
/// // Invalid client ip
/// client_subnet_encoding_data = parse_client_ip("1.2.a");
/// assert_eq!([0, 0, 0, 0, 0, 0, 0, 0], client_subnet_encoding_data.client_subnet.to_be_bytes());
/// assert_eq!(0, client_subnet_encoding_data.subnet_mask);
/// assert_eq!(0, client_subnet_encoding_data.is_ipv6);
/// ```
pub fn parse_client_ip(client_ip: &str) -> ClientSubnetEncodingData {
    if let Ok(addr) = client_ip.parse::<IpAddr>() {
        if addr.is_ipv4() {
            // unwrap is ok here because we verify it is parsable before
            let ipv4_address: Ipv4Addr = client_ip.parse().unwrap();
            ClientSubnetEncodingData {
                client_subnet: (u32::from_be_bytes(ipv4_address.octets()) as u64 & 0xffffff00)
                    << 32,
                subnet_mask: SubnetMask::Ipv4 as u64,
                is_ipv6: 0,
            }
        } else {
            // unwrap is ok here because we verify it is parsable before
            let ipv6_address: Ipv6Addr = client_ip.parse().unwrap();
            ClientSubnetEncodingData {
                client_subnet: ((u128::from_be_bytes(ipv6_address.octets()) >> 64)
                    & 0xffffffffffff0000) as u64,
                subnet_mask: SubnetMask::Ipv6 as u64,
                is_ipv6: 1,
            }
        }
    } else {
        ClientSubnetEncodingData {
            client_subnet: 0,
            subnet_mask: 0,
            is_ipv6: 0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::parse_client_ip;

    #[test]
    fn validate_parse_ipv4() {
        let client_subnet_encoding_data = parse_client_ip("85.83.215.126");

        assert_eq!(
            6148494311290830848,
            client_subnet_encoding_data.client_subnet
        );
        assert_eq!(24, client_subnet_encoding_data.subnet_mask);
        assert_eq!(0, client_subnet_encoding_data.is_ipv6);
    }

    #[test]
    fn validate_parse_ipv6() {
        let client_subnet_encoding_data =
            parse_client_ip("819e:5c2e:21e4:0094:4805:1635:f8e4:049b");

        assert_eq!(
            9340004030419828736,
            client_subnet_encoding_data.client_subnet
        );
        assert_eq!(48, client_subnet_encoding_data.subnet_mask);
        assert_eq!(1, client_subnet_encoding_data.is_ipv6);
    }

    #[test]
    fn validate_parse_abbreviated_ipv6() {
        let client_subnet_encoding_data = parse_client_ip("0319:7db1:f4d6::");

        assert_eq!(
            223347859801899008,
            client_subnet_encoding_data.client_subnet
        );
        assert_eq!(48, client_subnet_encoding_data.subnet_mask);
        assert_eq!(1, client_subnet_encoding_data.is_ipv6);
    }

    #[test]
    fn validate_parse_invalid_client_ip() {
        let client_subnet_encoding_data = parse_client_ip("1.2");

        assert_eq!(0, client_subnet_encoding_data.client_subnet);
        assert_eq!(0, client_subnet_encoding_data.subnet_mask);
        assert_eq!(0, client_subnet_encoding_data.is_ipv6);
    }
}
