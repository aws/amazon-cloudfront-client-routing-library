// Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0

pub fn get_mask(num_bits: u8) -> u64 {
    ((1_u128 << num_bits) - 1) as u64
}