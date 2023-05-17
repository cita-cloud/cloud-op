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

mod eth;
mod sm;

pub enum CryptoType {
    Sm,
    Eth,
}

impl From<&str> for CryptoType {
    fn from(str: &str) -> Self {
        match str.to_lowercase().as_str() {
            "sm" => CryptoType::Sm,
            "eth" => CryptoType::Eth,
            _ => panic!("crypto type only sm or eth"),
        }
    }
}
