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

use std::path::PathBuf;
use std::sync::Arc;

use crate::config::ConsensusType;
use crate::config::ExecutorConfig;
use crate::crypto::CryptoType;
use cita_database::{Config, DataCategory, Database, RocksDB, NUM_COLUMNS};
use cita_trie::{PatriciaTrie, Trie, DB};
use cita_types::{Address, H256};
use cita_vm::common;
use cita_vm::state::{AccountDB, StateObject};
use executor_evm::trie_db::{NodeType, TrieDb};
use executor_evm::types::db_indexes::{BlockNumber2Hash, CurrentHash, DbIndex, Hash2Header};
use executor_evm::types::header::Header;
use rlp::decode;

pub fn state_backup_inner(
    config_path: PathBuf,
    backup_path: PathBuf,
    height: u64,
    _consensus: ConsensusType,
    _crypto: CryptoType,
) {
    if let Err(e) = std::fs::create_dir_all(&backup_path) {
        println!(" backup dir create err {:?}", e);
        return;
    }

    let hasher = Arc::new(common::hash::get_hasher());

    let config = Config::with_category_num(NUM_COLUMNS);
    let executor_config = ExecutorConfig::new(config_path.to_str().unwrap());
    let statedb_path = executor_config.db_path + "/statedb";
    let state_rocks_db = Arc::new(RocksDB::open(&statedb_path, &config).unwrap());

    let backup_path = backup_path.join(height.to_string());
    let backup_rocks_db = Arc::new(RocksDB::open(backup_path.to_str().unwrap(), &config).unwrap());

    // get block hash
    let height_key = BlockNumber2Hash(height).get_index();
    let block_hash = state_rocks_db
        .get(Some(DataCategory::Extra), &height_key.to_vec())
        .unwrap_or(None)
        .map(|h| decode::<H256>(&h).unwrap())
        .unwrap();
    let block_header = state_rocks_db
        .get(Some(DataCategory::Headers), block_hash.as_bytes())
        .unwrap_or(None)
        .map(|header| decode::<Header>(header.as_slice()).unwrap())
        .unwrap();
    let state_root = block_header.state_root();

    let block_hash_value = rlp::encode(&block_hash).to_vec();
    backup_rocks_db
        .insert(
            Some(DataCategory::Extra),
            CurrentHash.get_index().to_vec(),
            block_hash_value,
        )
        .unwrap();

    backup_extra(&state_rocks_db, &backup_rocks_db, height);

    let state_db = Arc::new(TrieDb::new(Arc::clone(&state_rocks_db), NodeType::Archive));
    let backup_db = Arc::new(TrieDb::new(Arc::clone(&backup_rocks_db), NodeType::Full));

    let (pt, addrs) = PatriciaTrie::extract_backup(
        Arc::clone(&state_db),
        Some(Arc::clone(&backup_db)),
        Arc::clone(&hasher),
        &state_root.0,
    )
    .unwrap();

    for addr in addrs {
        let st_data = pt.get(&addr).unwrap().unwrap();
        let addr = Address::from_slice(addr.as_slice());
        // get account state object from vm
        let st_obj = StateObject::from_rlp(&st_data).unwrap();

        let state_db = Arc::new(AccountDB::new(
            addr,
            Arc::new(TrieDb::new(Arc::clone(&state_rocks_db), NodeType::Archive)),
        ));
        let backup_db = Arc::new(AccountDB::new(
            addr,
            Arc::new(TrieDb::new(Arc::clone(&backup_rocks_db), NodeType::Full)),
        ));
        // store account's code & abi
        if st_obj.code_hash != common::hash::NIL_DATA {
            let code = state_db.get(st_obj.code_hash.as_bytes()).unwrap().unwrap();
            backup_db.insert(st_obj.code_hash.0.to_vec(), code).unwrap();
        }
        if st_obj.abi_hash != common::hash::NIL_DATA {
            let abi = state_db.get(st_obj.abi_hash.as_bytes()).unwrap().unwrap();
            backup_db.insert(st_obj.abi_hash.0.to_vec(), abi).unwrap();
        }

        // store account's state
        PatriciaTrie::extract_backup(
            Arc::clone(&state_db),
            Some(Arc::clone(&backup_db)),
            Arc::clone(&hasher),
            st_obj.storage_root.as_bytes(),
        )
        .unwrap();
    }
    println!("extract_backup finish.");
}

fn backup_extra(state_rocks_db: &Arc<RocksDB>, backup_rocks_db: &Arc<RocksDB>, height: u64) {
    for i in 0..=height {
        let height_key = BlockNumber2Hash(height - i).get_index();
        let block_hash = state_rocks_db
            .get(Some(DataCategory::Extra), &height_key.to_vec())
            .unwrap_or(None)
            .map(|h| decode::<H256>(h.as_slice()).unwrap())
            .unwrap();
        let block_header = state_rocks_db
            .get(Some(DataCategory::Headers), block_hash.as_bytes())
            .unwrap_or(None)
            .map(|header| decode::<Header>(header.as_slice()).unwrap())
            .unwrap();

        let block_hash_value = rlp::encode(&block_hash).to_vec();
        backup_rocks_db
            .insert(
                Some(DataCategory::Extra),
                height_key.to_vec(),
                block_hash_value,
            )
            .unwrap();
        backup_rocks_db
            .insert(
                Some(DataCategory::Headers),
                Hash2Header(block_hash).get_index().to_vec(),
                block_header.rlp(),
            )
            .unwrap();
    }
}
