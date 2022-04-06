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

use cita_database::{Config, DataCategory, Database, RocksDB, NUM_COLUMNS};
use executor_evm::{
    config::ExecutorConfig,
    types::{
        db_indexes::{BlockNumber2Hash, BlockNumber2Header, CurrentHash, DbIndex},
        header::Header,
        H256,
    },
};
use fs_extra::{copy_items, dir};
use std::fs::remove_dir_all;
use std::path::Path;

pub fn executor_recover(config_path: &Path, height: u64) {
    let executor_config = ExecutorConfig::new(config_path.to_str().unwrap());
    let state_path = executor_config.db_path.clone() + "/statedb";
    state_recover(&state_path, height);

    let chain_path = executor_config.db_path + "/nosql";
    chain_db_recover(&chain_path, height);
}

fn state_recover(state_path: &str, height: u64) {
    if !Path::new(&state_path).exists() {
        panic!("executor state_db dir not exist");
    }

    let database_config = Config::with_category_num(NUM_COLUMNS);
    let exec_db = RocksDB::open(state_path, &database_config).unwrap();

    let pkey = BlockNumber2Hash(height).get_index().to_vec();
    let dst_hash = exec_db
        .get(Some(DataCategory::Extra), &pkey)
        .unwrap_or(None)
        .map(|h| rlp::decode::<H256>(&h).unwrap())
        .unwrap();

    exec_db
        .insert(
            Some(DataCategory::Extra),
            CurrentHash.get_index().to_vec(),
            rlp::encode(&dst_hash).to_vec(),
        )
        .unwrap();
}

fn chain_db_recover(chain_path: &str, height: u64) {
    if !Path::new(chain_path).exists() {
        panic!("executor chain db dir not exist");
    }

    let database_config = Config::with_category_num(NUM_COLUMNS);
    let chain_db = RocksDB::open(chain_path, &database_config).expect("DB file not found");

    let hkey = BlockNumber2Header(height).get_index().to_vec();

    let dst_header = chain_db
        .get(Some(DataCategory::Headers), &hkey)
        .unwrap_or(None)
        .map(|hdr| rlp::decode::<Header>(&hdr).unwrap())
        .unwrap();

    let dst_hash = dst_header.hash().unwrap();

    chain_db
        .insert(
            Some(DataCategory::Extra),
            CurrentHash.get_index().to_vec(),
            rlp::encode(&dst_hash).to_vec(),
        )
        .unwrap();
}

pub fn move_state(config_path: &Path, backup_path: &Path, height: u64) {
    let snap_path = backup_path.join(height.to_string());
    let executor_config = ExecutorConfig::new(config_path.to_str().unwrap());
    let state_path = executor_config.db_path.clone() + "/statedb";

    let _ = remove_dir_all(&state_path);

    let mut copy_option = dir::CopyOptions::new();
    copy_option.copy_inside = true;
    copy_items(&[snap_path], state_path, &copy_option).unwrap();
}
