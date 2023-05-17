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

use storage_opendal::config::StorageConfig;
use storage_opendal::storager::Storager;

use crate::config::ConsensusType;
use crate::crypto::CryptoType;
use crate::recover::chain::chain_recover;
pub use crate::recover::common_storage::common_storage_recover;
use crate::recover::executor::{executor_recover, move_state};
use crate::recover::utxo::utxo_recover;
use std::path::PathBuf;

pub async fn recover(
    config_path: PathBuf,
    height: u64,
    consensus: ConsensusType,
    _crypto: CryptoType,
    clear_consensus_data: bool,
) {
    let config = StorageConfig::new(config_path.to_str().unwrap());
    let db = Storager::build(
        &config.data_root,
        &config.cloud_storage,
        config.l1_capacity,
        config.l2_capacity,
        u64::MAX,
        config.retreat_interval,
    );

    // recover chain db
    chain_recover(&db, &config_path, height, consensus, clear_consensus_data).await;
    // recover executor
    executor_recover(&config_path, height);
    // recover utxo
    utxo_recover(&db, &config_path, height).await;
}

pub async fn state_recover(
    config_path: PathBuf,
    backup_path: PathBuf,
    height: u64,
    consensus: ConsensusType,
    _crypto: CryptoType,
    clear_consensus_data: bool,
) {
    let config = StorageConfig::new(config_path.to_str().unwrap());
    let db = Storager::build(
        &config.data_root,
        &config.cloud_storage,
        config.l1_capacity,
        config.l2_capacity,
        config.backup_interval,
        config.retreat_interval,
    );
    // recover chain db
    chain_recover(&db, &config_path, height, consensus, clear_consensus_data).await;
    // recover executor from specify state
    move_state(&config_path, &backup_path, height);
    // recover utxo
    utxo_recover(&db, &config_path, height).await;
}

pub fn get_real_key(region: u32, key: &[u8]) -> String {
    hex::encode([region.to_be_bytes().as_slice(), key].concat())
}

mod chain;
mod common_storage;
mod executor;
mod utxo;
