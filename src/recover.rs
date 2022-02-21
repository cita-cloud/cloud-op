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

use crate::recover::chain::chain_recover;
use crate::recover::executor::executor_recover;
use crate::recover::utxo::utxo_recover;
use std::path::PathBuf;

pub fn recover(config_path: PathBuf, height: u64) {
    // recover chain db
    chain_recover(&config_path, height);
    // recover executor
    executor_recover(&config_path, height);
    // recover utxo
    utxo_recover(&config_path, height);
    // recover executor
}

mod chain;
mod executor;
mod utxo;
