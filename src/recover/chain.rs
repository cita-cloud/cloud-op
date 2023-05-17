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

use crate::config::{ConsensusType, OverlordConsensusConfig, RaftConsensusConfig};
use crate::recover::get_real_key;
use crate::storage::Storager;
use std::fs::remove_dir_all;
use std::path::Path;

pub async fn chain_recover(
    db: &Storager,
    config_path: &Path,
    height: u64,
    consensus: ConsensusType,
    clear_consensus_data: bool,
) {
    let current_height_bytes = db
        .load(&get_real_key(0, &0u64.to_be_bytes()), true)
        .await
        .unwrap();
    let mut buf: [u8; 8] = [0; 8];
    buf.clone_from_slice(&current_height_bytes[..8]);
    let current_height = u64::from_be_bytes(buf);
    println!("layer2 current height: {}", current_height);
    println!("layer2 current height recover to: {}", height);

    if height >= current_height {
        panic!(
            "recover height({}) >= current height({})",
            height, current_height
        );
    }

    // remove consensus wal file
    if clear_consensus_data {
        match consensus {
            ConsensusType::Raft => {
                let consensus_config = RaftConsensusConfig::new(config_path.to_str().unwrap());
                let _ = remove_dir_all(consensus_config.wal_path);
            }
            ConsensusType::Overlord => {
                let consensus_config = OverlordConsensusConfig::new(config_path.to_str().unwrap());
                let _ = remove_dir_all(consensus_config.wal_path);
            }
        }
    }

    // recover current height and delete height
    db.store(
        &get_real_key(0, &0u64.to_be_bytes()),
        height.to_be_bytes().to_vec(),
    )
    .await
    .unwrap();

    let res = db.load(&get_real_key(0, &2u64.to_be_bytes()), true).await;
    if let Ok(delete_height_bytes) = res {
        let mut buf: [u8; 8] = [0; 8];
        buf.clone_from_slice(&delete_height_bytes[..8]);
        let delete_height = u64::from_be_bytes(buf);
        println!("layer2 current delete_height: {}", delete_height);
        let new_delete_height = height.min(delete_height);
        println!("layer2 delete_height recover to: {}", new_delete_height);

        db.store(
            &get_real_key(0, &2u64.to_be_bytes()),
            new_delete_height.to_be_bytes().to_vec(),
        )
        .await
        .unwrap();
    }
}
