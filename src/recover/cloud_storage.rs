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

use super::get_real_key;
use crate::storage::{StorageConfig, Storager};
use std::path::Path;

pub async fn cloud_storage_recover(config_path: &Path, recover_backup_height: u64) {
    let config = StorageConfig::new(config_path.to_str().unwrap());
    let db = Storager::build(
        &config.data_root,
        &config.cloud_storage,
        config.l1_capacity,
        config.l2_capacity,
        config.backup_interval,
        config.retreat_interval,
    )
    .await;

    let remote = &db
        .next_storager
        .as_ref()
        .unwrap()
        .next_storager
        .as_ref()
        .unwrap()
        .operator;

    if let Ok(remote_height_bytes) = remote.read(&get_real_key(0, &1u64.to_be_bytes())).await {
        let mut buf: [u8; 8] = [0; 8];
        buf.clone_from_slice(&remote_height_bytes[..8]);
        let current_backup_height = u64::from_be_bytes(buf);
        if recover_backup_height >= current_backup_height {
            panic!(
                "recover backup_height({}) >= current backup_height({})",
                recover_backup_height, current_backup_height
            );
        }

        let local = &db.next_storager.as_ref().unwrap().operator;
        if let Ok(local_delete_bytes) = local.read(&get_real_key(0, &2u64.to_be_bytes())).await {
            let mut buf: [u8; 8] = [0; 8];
            buf.clone_from_slice(&local_delete_bytes[..8]);
            let local_delete_height = u64::from_be_bytes(buf);
            if recover_backup_height < local_delete_height {
                panic!(
                    "recover_backup_height({}) < local_delete_height({})",
                    recover_backup_height, local_delete_height
                );
            }
        }

        remote
            .write(
                &get_real_key(0, &1u64.to_be_bytes()),
                recover_backup_height.to_be_bytes().to_vec(),
            )
            .await
            .unwrap();
    } else {
        println!("backup hasn't started");
    }
}
