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

use crate::util::{get_real_key, storage_db, StorageDb};
use std::path::Path;

pub async fn cloud_storage_rollback(config_path: &Path, rollback_backup_height: u64) {
    let storage_db = storage_db(config_path).await;
    let StorageDb::Opendal(storager) = &storage_db else {
        panic!("cloud rollback not support rocksdb")
    };

    let remote = &storager
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
        println!("current_backup_height: {current_backup_height}");
        if rollback_backup_height >= current_backup_height {
            panic!(
                "rollback backup_height({}) >= current backup_height({})",
                rollback_backup_height, current_backup_height
            );
        }
        println!("rollback_backup_height: {rollback_backup_height}");

        remote
            .write(
                &get_real_key(0, &1u64.to_be_bytes()),
                rollback_backup_height.to_be_bytes().to_vec(),
            )
            .await
            .unwrap();
        println!("cloud rollback done!");
    } else {
        println!("backup hasn't started");
    }
}
