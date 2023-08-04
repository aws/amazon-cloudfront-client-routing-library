#[cfg(test)]
mod test_encode_request_data {
    use amazon_cloudfront_client_routing_lib::encode_request_data;

    #[test]
    fn validate_encode_with_ipv4() {
        let encoded_label = encode_request_data("85.83.215.126", "B086VX9VMK", "example.com");

        assert_eq!("abfku6xaaaaaaaamotptyubibrji6.example.com", encoded_label);
    }

    #[test]
    fn validate_encode_with_ipv6() {
        let encoded_label = encode_request_data(
            "819e:5c2e:21e4:0094:4805:1635:f8e4:049b",
            "Q9OP1I23",
            "example.com",
        );

        assert_eq!("abydhs4fyq6iaaaykudpmaxncecqs.example.com", encoded_label);
    }

    #[test]
    fn validate_encode_with_abbreviated_ipv6() {
        let encoded_label = encode_request_data("2c0f:f386:9f5b:a3ad::", "ZZAA12TP", "example.com");

        assert_eq!("absyd7tq2pvwaaayipu4qwb2rlz4g.example.com", encoded_label);
    }

    #[test]
    fn validate_encode_with_subdomain_in_fqdn() {
        let encoded_label = encode_request_data(
            "122.71.138.53",
            "12PC5GH7Y0ABCDEFGHIJHJUIOZZAA1",
            "test.example2.com",
        );

        assert_eq!(
            "abhur4kaaaaaaaampbtn52pincn7x.test.example2.com",
            encoded_label
        );
    }

    #[test]
    fn validate_encode_with_path_in_fqdn() {
        let encoded_label = encode_request_data(
            "0319:7db1:f4d6:62ec:10cf:ffe4:4270:d2d5",
            "AC2Q2389",
            "example.com/movie/12ab4c?query=watch",
        );

        assert_eq!(
            "abqggl5wh2nmaaaypv4i33wdvvtdk.example.com/movie/12ab4c?query=watch",
            encoded_label
        );
    }

    #[test]
    fn validate_encode_with_invalid_client_ip() {
        let encoded_label = encode_request_data("122.71", "DP0124QHYT", "example.com");

        assert_eq!("abaaaaaaaaaaaaaaoqysz2z3j45da.example.com", encoded_label);
    }

    #[test]
    fn validate_encode_with_no_cgid() {
        let encoded_label = encode_request_data("46.3.3.135", "", "example.com");

        assert_eq!("abc4aydaaaaaaaamaaaaaaaaaaaaa.example.com", encoded_label);
    }

    #[test]
    fn validate_encode_with_no_fqdn() {
        let encoded_label =
            encode_request_data("6687:1cc9:0e87:2b33:1181:eff2:9a6a:786b", "DF97B6J1O0", "");

        assert_eq!("abwnby4zehioaaaymv5p6exntn7z3.", encoded_label);
    }

    #[test]
    fn validate_encode_with_no_client_ip_no_cgid() {
        let encoded_label = encode_request_data("", "", "example.com");

        assert_eq!("abaaaaaaaaaaaaaaaaaaaaaaaaaaa.example.com", encoded_label);
    }

    #[test]
    fn validate_encode_with_no_client_ip_no_cgid_no_fqdn() {
        let encoded_label = encode_request_data("", "", "");

        assert_eq!("abaaaaaaaaaaaaaaaaaaaaaaaaaaa.", encoded_label);
    }
}
