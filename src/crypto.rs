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

#[cfg(feature = "crypto_sm")]
mod sm;
#[cfg(feature = "crypto_sm")]
pub use sm::{hash_data, pk2address};

#[cfg(feature = "crypto_eth")]
mod eth;
#[cfg(feature = "crypto_eth")]
pub use eth::{hash_data, pk2address};
