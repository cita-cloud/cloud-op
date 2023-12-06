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

mod state_backup;
use self::state_backup::state_snapshot_backup;
use crate::{
    rollback::{executor_rollback, storage_rollback},
    util::{copy_dir, executor_db_path, get_real_key, read_current_height, storage_db, StorageDb},
};
use cita_cloud_proto::storage::Regions;
use std::path::{Path, PathBuf};
use storage_opendal::{config::StorageConfig as OpendalConfig, storager::Storager};

pub async fn backup(
    config_path: PathBuf,
    mut backup_path: PathBuf,
    height: Option<u64>,
    snapshot_only: bool,
    old_storage: bool,
) {
    // check height
    let storage_db = storage_db(&config_path, old_storage).await;
    let executor_db_path = executor_db_path(&config_path);
    let current_height = read_current_height(&storage_db).await;
    println!("current height: {}", current_height);
    let backup_height = height.unwrap_or(current_height);
    println!("backup height: {}", backup_height);
    if backup_height > current_height {
        panic!(
            "backup height({}) > current height({})",
            backup_height, current_height
        );
    }
    // create backup dir
    backup_path = backup_path.join(backup_height.to_string());

    // backup executor state
    let state_path = executor_db_path.to_owned() + "/statedb";
    let state_backup_path = backup_path.clone().join("data/statedb");
    state_snapshot_backup(&state_path, &state_backup_path, backup_height);
    println!("backup excutor state done!");

    if !snapshot_only {
        // backup executor chain_db
        let chain_path = executor_db_path.to_owned() + "/nosql";
        let executor_backup_path = backup_path.clone().join("data/nosql");
        copy_dir(Path::new(&chain_path), &executor_backup_path);
        println!("copy excutor chain_db done!");

        // backup storage chain_data
        if old_storage {
            let StorageDb::RocksDB(read) = &storage_db else {
                unreachable!()
            };
            let storage_backup_path = backup_path.clone().join("chain_data");
            let config = OpendalConfig::default();
            let write = Storager::build(
                storage_backup_path.to_str().unwrap(),
                &config.cloud_storage,
                config.l1_capacity,
                config.l2_capacity,
                u32::MAX as u64,
                config.retreat_interval,
            )
            .await;
            for height in 0..=current_height {
                print!("\rconverting old: {}/{}", height, current_height);
                let height_bytes = height.to_be_bytes().to_vec();
                let old_block_bytes = read.load_full_block(height_bytes.clone()).unwrap();
                let mut old_bytes = read.load(4, height_bytes.clone()).unwrap();
                old_bytes.extend_from_slice(&old_block_bytes);

                write
                    .store_all_block_data(&height_bytes, &old_bytes)
                    .await
                    .unwrap();
            }
            println!("\nconvert old block done!");

            let global_region = i32::from(Regions::Global) as u32;
            for lock_id in 1000u64..1008 {
                if let Ok(hash) = read.load(global_region, lock_id.to_be_bytes().to_vec()) {
                    write
                        .store(
                            &get_real_key(global_region, lock_id.to_be_bytes().as_slice()),
                            hash.as_slice(),
                        )
                        .await
                        .unwrap();
                }
            }
            println!("convert old utxo done!");
        } else {
            let StorageDb::Opendal(storager) = &storage_db else {
                unreachable!()
            };
            let storage_path = storager
                .next_storager
                .as_ref()
                .unwrap()
                .operator
                .info()
                .name()
                .to_owned();
            let storage_backup_path = backup_path.clone().join("chain_data");
            copy_dir(Path::new(&storage_path), &storage_backup_path);
            println!("copy storage chain_data done!");
        }

        if backup_height != current_height {
            let storage_backup_path = backup_path.clone().join("chain_data");
            let config = OpendalConfig::default();
            let write = Storager::build(
                storage_backup_path.to_str().unwrap(),
                &config.cloud_storage,
                config.l1_capacity,
                config.l2_capacity,
                u32::MAX as u64,
                config.retreat_interval,
            )
            .await;
            // rollback to backup_height
            executor_rollback(&executor_db_path, backup_height);
            storage_rollback(&write, backup_height, false).await;
        }
    }
    println!("backup done!");
}
