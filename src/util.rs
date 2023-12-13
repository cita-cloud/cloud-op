// Copyright Rivtower Technologies LLC.
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//:q
// http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use executor_evm::config::ExecutorConfig;
use fs_extra::{copy_items, dir::CopyOptions};
use std::fs;
use std::path::Path;
use storage_opendal::{config::StorageConfig as OpendalConfig, storager::Storager};
use storage_rocksdb::{config::StorageConfig as RocksdbConfig, db::DB};
use toml::Table;

pub const HASH_LEN: u32 = 32;

pub fn get_real_key(region: u32, key: &[u8]) -> String {
    hex::encode([region.to_be_bytes().as_slice(), key].concat())
}

pub fn executor_db_path(config_path: &Path) -> String {
    ExecutorConfig::new(config_path.to_str().unwrap()).db_path
}

pub enum StorageDb {
    RocksDB(DB),
    Opendal(Storager),
}

pub async fn storage_db(config_path: &Path) -> StorageDb {
    let s = fs::read_to_string(config_path)
        .map_err(|e| println!("read config err: {e}"))
        .unwrap();
    let config: Table = s
        .parse::<Table>()
        .map_err(|e| println!("config toml parse err: {e}"))
        .unwrap();

    if config.contains_key("storage_rocksdb") {
        let config = RocksdbConfig::new(config_path.to_str().unwrap());
        StorageDb::RocksDB(DB::new(&config.db_path, &config))
    } else if config.contains_key("storage_opendal") {
        let config = OpendalConfig::new(config_path.to_str().unwrap());
        StorageDb::Opendal(
            Storager::build(
                &config.data_root,
                &config.cloud_storage,
                config.l1_capacity,
                config.l2_capacity,
                u32::MAX as u64,
                config.retreat_interval,
            )
            .await,
        )
    } else {
        panic!("storage config not found")
    }
}

pub async fn read_current_height(storager: &StorageDb) -> u64 {
    let current_height_bytes = match storager {
        StorageDb::RocksDB(db) => db.load(0, 0u64.to_be_bytes().to_vec()).unwrap(),
        StorageDb::Opendal(storager) => storager
            .load(&get_real_key(0, &0u64.to_be_bytes()), true)
            .await
            .unwrap(),
    };
    let mut buf: [u8; 8] = [0; 8];
    buf.clone_from_slice(&current_height_bytes[..8]);
    u64::from_be_bytes(buf)
}

pub fn copy_dir(source_path: &Path, target_path: &Path) {
    if !source_path.exists() {
        panic!("source_path not exist")
    }

    let mut copy_option = CopyOptions::new();
    copy_option.copy_inside = true;
    copy_option.overwrite = true;
    copy_items(&[source_path], target_path, &copy_option).unwrap();
}
