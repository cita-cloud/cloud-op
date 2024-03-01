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

use crate::{
    rollback::{executor_rollback, rocksdb_rollback, storage_rollback},
    util::{copy_dir, executor_db_path, read_current_height, storage_db},
};
use std::path::{Path, PathBuf};
use storage_opendal::{config::StorageConfig as OpendalConfig, storager::Storager};
use storage_rocksdb::{config::StorageConfig as RocksdbConfig, db::DB};

use std::fs;
use toml::Table;

pub async fn backup(config_path: PathBuf, mut backup_path: PathBuf, height: Option<u64>) {
    // check height
    let storage_db = storage_db(&config_path).await;
    let executor_db_path = executor_db_path(&config_path);
    let current_height = read_current_height(&storage_db).await;
    println!("current height: {}", current_height);
    let backup_height = height.unwrap_or(current_height);
    println!("backup height: {}", backup_height);
    if backup_height >= current_height {
        panic!(
            "backup height({}) >= current height({})",
            backup_height, current_height
        );
    }
    // create backup dir
    backup_path = backup_path.join(backup_height.to_string());

    // backup executor state
    let state_path = executor_db_path.to_owned() + "/statedb";
    let state_backup_path = backup_path.clone().join("data/statedb");
    copy_dir(Path::new(&state_path), &state_backup_path);
    println!("copy excutor state done!");

    // backup executor chain_db
    let chain_path = executor_db_path.to_owned() + "/nosql";
    let executor_backup_path = backup_path.clone().join("data/nosql");
    copy_dir(Path::new(&chain_path), &executor_backup_path);
    println!("copy excutor chain_db done!");

    // backup storage data
    // directly copy storage data
    let s = fs::read_to_string(&config_path)
        .map_err(|e| println!("read config err: {e}"))
        .unwrap();
    let config: Table = s
        .parse::<Table>()
        .map_err(|e| println!("config toml parse err: {e}"))
        .unwrap();
    let storage_path = if config.contains_key("storage_rocksdb") {
        let config = RocksdbConfig::new(config_path.to_str().unwrap());
        config.db_path
    } else if config.contains_key("storage_opendal") {
        let config = OpendalConfig::new(config_path.to_str().unwrap());
        config.data_root
    } else {
        panic!("storage config not found")
    };
    let storage_backup_path = backup_path.clone().join("chain_data");
    copy_dir(Path::new(&storage_path), &storage_backup_path);
    println!("copy storage chain_data done!");

    // rollback to backup_height
    executor_rollback(
        backup_path
            .clone()
            .join("data")
            .as_os_str()
            .to_str()
            .unwrap(),
        backup_height,
    );
    let storage_backup_path = backup_path.clone().join("chain_data");

    if config.contains_key("storage_opendal") {
        let storage_config = OpendalConfig::default();
        let write = Storager::build(
            storage_backup_path.to_str().unwrap(),
            &storage_config.cloud_storage,
            storage_config.l1_capacity,
            storage_config.l2_capacity,
            u32::MAX as u64,
            storage_config.retreat_interval,
        )
        .await;
        storage_rollback(&write, backup_height, false).await;
    } else if config.contains_key("storage_rocksdb") {
        let storage_config = RocksdbConfig::default();
        let db = DB::new(storage_backup_path.to_str().unwrap(), &storage_config);
        rocksdb_rollback(&db, backup_height, false, false);
    } else {
        panic!("storage config not found")
    };
    println!("backup done!");
}
