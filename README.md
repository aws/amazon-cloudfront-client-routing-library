## Amazon CloudFront Client Routing Library

The Amazon CloudFront Client Routing Library is an open-source library designed for CloudFront's Client Routing feature, which is used direct client devices to CloudFront Points of Presence (POPs) with greater precision. Client Routing feature utilizes information present within a specially formatted DNS label, and this library provides functions to encode and decode such DNS labels.


### What is Client Routing?

Client Routing is a new feature from CloudFront which utilizes client subnet information encoded in a DNS label to route traffic to CloudFront POPs. In addition to utilizing this library, this feature has associated prerequisites such as using Route53 and requiring certificate updates. Documentation for Client Routing will be released in the near future, but if you are interested in using this feature sooner, please reach out to AWS Support to know more.


### How to Use the Amazon CloudFront Client Routing Library?

There are two main functions in the library: `encode_request_data` and `decode_request_data`.

#### Encoding

`encode_request_data` takes three parameters: `client_ip`, `content_group_id`, and `fqdn`. A Client Routing label is generated from this data and then that label is returned prepended as a subdomain to the `fqdn`.

The `content_group_id` is set aside for future use and must be set to an empty string for now.


```
let encoded_label = amazon_cloudfront_client_routing_lib::encode_request_data("1.2.3.4", "", "example.com"); // encoded_label is abacaqdaaaaaaaamaaaaaaaaaaaaa.example.com
```

#### Decoding

`decode_request_data` takes one parameter: `domain`. A result containing either a `DecodedClientRoutingLabel` struct or a `DecodeLengthError` is returned with each field set according to the `domain`. The `domain` can be either a FQDN or just the Client Routing label.

```
let decoded_label = amazon_cloudfront_client_routing_lib::decode_request_data("abacaqdaaaaaaaamaaaaaaaaaaaaa").unwrap();
// DecodedClientRoutingLabel {
//     client_sdk_version: 1,
//     is_ipv6: false,
//     client_subnet: [1, 2, 3, 0, 0, 0, 0, 0],
//     subnet_mask: 24,
//     cgid: 0,
// }
let decoded_label = amazon_cloudfront_client_routing_lib::decode_request_data("abacaqdaaaaaaaamaaaaaaaaaaaaa.example.com").unwrap();
// DecodedClientRoutingLabel {
//     client_sdk_version: 1,
//     is_ipv6: false,
//     client_subnet: [1, 2, 3, 4, 0, 0, 0, 0],
//     subnet_mask: 24,
//     cgid: 0,
// }
```

If the first dns label of the domain is an invalid Client Routing label (eg. improper length) then the result will contain an error.

```
let decoded_label = amazon_cloudfront_client_routing_lib::decode_request_data("abacaqdaaaaaaaamnjg3oubcyv").unwrap();
// DecodeLengthError {
//     num_chars: 26,
//     expected_num_chars: 29   
// }
```

## License

This library is licensed under the Apache 2.0 License.
