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

use crate::config::ControllerConfig;
use crate::config::{BftConsensusConfig, ConsensusType, RaftConsensusConfig};
use crate::storage::{StorageConfig, DB};
use cita_cloud_proto::blockchain::CompactBlock;
use prost::Message;
use std::fs::remove_dir_all;
use std::path::Path;

pub fn chain_recover(config_path: &Path, height: u64, consensus: ConsensusType) {
    let storage_config = StorageConfig::new(config_path.to_str().unwrap());
    let db = DB::new(&storage_config.db_path, &storage_config);

    let current_height_bytes = db.load(0, 0u64.to_be_bytes().to_vec()).unwrap();
    let mut buf: [u8; 8] = [0; 8];
    buf.clone_from_slice(&current_height_bytes[..8]);
    let current_height = u64::from_be_bytes(buf);

    if height >= current_height {
        panic!(
            "specify height({}) > current height({})",
            height, current_height
        );
    }

    // remove controller wal file
    let controller_config = ControllerConfig::new(config_path.to_str().unwrap());
    let _ = remove_dir_all(&controller_config.wal_path);

    // remove consensus wal file
    match consensus {
        ConsensusType::Bft => {
            let consensus_config = BftConsensusConfig::new(config_path.to_str().unwrap());
            let _ = remove_dir_all(&consensus_config.wal_path);
        }
        ConsensusType::Raft => {
            let consensus_config = RaftConsensusConfig::new(config_path.to_str().unwrap());
            let _ = remove_dir_all(&consensus_config.wal_path);
        }
    }

    let height_bytes = (height + 1).to_be_bytes().to_vec();
    let compact_block_bytes = db.load(10, height_bytes).unwrap();
    let compact_block = CompactBlock::decode(compact_block_bytes.as_slice()).unwrap();

    let hash = compact_block.header.unwrap().prevhash;

    // recover current height & hash
    db.store(
        0,
        0u64.to_be_bytes().to_vec(),
        height.to_be_bytes().to_vec(),
    )
    .unwrap();
    db.store(0, 1u64.to_be_bytes().to_vec(), hash).unwrap();
}
