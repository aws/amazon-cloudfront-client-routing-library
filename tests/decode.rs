#[cfg(test)]
mod test_encode_request_data {
    use amazon_cloudfront_client_routing_lib::decode_request_data;

    #[test]
    fn validate_decode_with_ipv4() {
        let decoded_label = match decode_request_data("abfku6xaaaaaaaamotptyubibrji6") {
            Ok(label) => label,
            Err(e) => panic!("{}", e),
        };

        assert_eq!(1, decoded_label.client_sdk_version);
        assert_eq!(16843032286346126622, decoded_label.cgid);
        assert_eq!(24, decoded_label.subnet_mask);
        assert_eq!(false, decoded_label.is_ipv6);
        assert_eq!([85, 83, 215, 0, 0, 0, 0, 0], decoded_label.client_subnet);
    }

    #[test]
    fn validate_decode_with_ipv6() {
        let decoded_label = match decode_request_data("abydhs4fyq6iaaaykudpmaxncecqs") {
            Ok(label) => label,
            Err(e) => panic!("{}", e),
        };

        assert_eq!(1, decoded_label.client_sdk_version);
        assert_eq!(12253709671023643154, decoded_label.cgid);
        assert_eq!(48, decoded_label.subnet_mask);
        assert_eq!(true, decoded_label.is_ipv6);
        assert_eq!(
            [0x81, 0x9e, 0x5c, 0x2e, 0x21, 0xe4, 0, 0],
            decoded_label.client_subnet
        );
    }

    #[test]
    fn validate_decode_with_fqdn() {
        let decoded_label = match decode_request_data("abydhs4fyq6iaaaykudpmaxncecqs.example.com") {
            Ok(label) => label,
            Err(e) => panic!("{}", e),
        };

        assert_eq!(1, decoded_label.client_sdk_version);
        assert_eq!(12253709671023643154, decoded_label.cgid);
        assert_eq!(48, decoded_label.subnet_mask);
        assert_eq!(true, decoded_label.is_ipv6);
        assert_eq!(
            [0x81, 0x9e, 0x5c, 0x2e, 0x21, 0xe4, 0, 0],
            decoded_label.client_subnet
        );

        let decoded_label = match decode_request_data("abfku6xaaaaaaaamotptyubibrji6.vod1.example.com") {
            Ok(label) => label,
            Err(e) => panic!("{}", e),
        };

        assert_eq!(1, decoded_label.client_sdk_version);
        assert_eq!(16843032286346126622, decoded_label.cgid);
        assert_eq!(24, decoded_label.subnet_mask);
        assert_eq!(false, decoded_label.is_ipv6);
        assert_eq!([85, 83, 215, 0, 0, 0, 0, 0], decoded_label.client_subnet);
    }

    #[test]
    fn validate_decode_with_no_client_subnet() {
        let decoded_label = match decode_request_data("abaaaaaaaaaaaaaaoqysz2z3j45da") {
            Ok(label) => label,
            Err(e) => panic!("{}", e),
        };

        assert_eq!(1, decoded_label.client_sdk_version);
        assert_eq!(16745045142164894816, decoded_label.cgid);
        assert_eq!(0, decoded_label.subnet_mask);
        assert_eq!(false, decoded_label.is_ipv6);
        assert_eq!([0, 0, 0, 0, 0, 0, 0, 0], decoded_label.client_subnet);
    }

    #[test]
    fn validate_decode_with_no_cgid() {
        let decoded_label = match decode_request_data("abc4aydaaaaaaaamaaaaaaaaaaaaa") {
            Ok(label) => label,
            Err(e) => panic!("{}", e),
        };

        assert_eq!(1, decoded_label.client_sdk_version);
        assert_eq!(0, decoded_label.cgid);
        assert_eq!(24, decoded_label.subnet_mask);
        assert_eq!(false, decoded_label.is_ipv6);
        assert_eq!([46, 3, 3, 0, 0, 0, 0, 0], decoded_label.client_subnet);
    }

    #[test]
    fn validate_decode_with_no_client_subnet_no_cgid() {
        let decoded_label = match decode_request_data("abaaaaaaaaaaaaaaaaaaaaaaaaaaa") {
            Ok(label) => label,
            Err(e) => panic!("{}", e),
        };

        assert_eq!(1, decoded_label.client_sdk_version);
        assert_eq!(0, decoded_label.cgid);
        assert_eq!(0, decoded_label.subnet_mask);
        assert_eq!(false, decoded_label.is_ipv6);
        assert_eq!([0, 0, 0, 0, 0, 0, 0, 0], decoded_label.client_subnet);
    }

    #[test]
    fn validate_decode_with_too_small_client_routing_label_returns_error() {
        match decode_request_data("abydhs4fyq6iaaaykudpmaxnce") {
            Ok(_dns_label) => {
                panic!("Didn't return an error when it should have")
            }
            Err(_e) => (),
        };
    }

    #[test]
    fn validate_decode_with_too_large_client_routing_label_returns_error() {
        match decode_request_data("abydhs4fyq6iaaaykudpmaxncecqsaaaa") {
            Ok(_dns_label) => {
                panic!("Didn't return an error when it should have")
            }
            Err(_e) => (),
        };
    }

    #[test]
    fn validate_decode_with_empty_domain_returns_error() {
        match decode_request_data("") {
            Ok(_dns_label) => {
                panic!("Didn't return an error when it should have")
            }
            Err(_e) => (),
        };
    }

    #[test]
    fn validate_decode_with_client_routing_label_not_first_dns_label_returns_error() {
        match decode_request_data("vod1.abfku6xaaaaaaaamotptyubibrji6.example.com") {
            Ok(_dns_label) => {
                panic!("Didn't return an error when it should have")
            }
            Err(_e) => (),
        };
    }
}
