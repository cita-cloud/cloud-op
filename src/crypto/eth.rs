// Copyright Rivtower Technologies LLC.
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
// http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

// This file is from kms_eth, and I made some modifications.

#![allow(dead_code)]

use tiny_keccak::{Hasher, Keccak};

pub const HASH_BYTES_LEN: usize = 32;
pub const ADDR_BYTES_LEN: usize = 20;

fn keccak_hash(input: &[u8]) -> [u8; HASH_BYTES_LEN] {
    let mut result = [0u8; HASH_BYTES_LEN];

    let mut keccak = Keccak::v256();
    keccak.update(input);
    keccak.finalize(&mut result);
    result
}

pub fn hash_data(data: &[u8]) -> Vec<u8> {
    keccak_hash(data).to_vec()
}

pub fn pk2address(pk: &[u8]) -> Vec<u8> {
    hash_data(pk)[HASH_BYTES_LEN - ADDR_BYTES_LEN..].to_vec()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keccak_test() {
        let hash_empty: [u8; HASH_BYTES_LEN] = [
            0xc5, 0xd2, 0x46, 0x01, 0x86, 0xf7, 0x23, 0x3c, 0x92, 0x7e, 0x7d, 0xb2, 0xdc, 0xc7,
            0x03, 0xc0, 0xe5, 0x00, 0xb6, 0x53, 0xca, 0x82, 0x27, 0x3b, 0x7b, 0xfa, 0xd8, 0x04,
            0x5d, 0x85, 0xa4, 0x70,
        ];
        assert_eq!(keccak_hash(&[]), hash_empty);
    }
}
