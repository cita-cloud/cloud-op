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
use crate::storage::{StorageConfig, DB};
use cita_cloud_proto::blockchain::raw_transaction::Tx::UtxoTx;
use cita_cloud_proto::blockchain::RawTransaction;
use prost::Message;
use std::path::Path;

pub const LOCK_ID_VERSION: u64 = 1_000;
pub const LOCK_ID_CHAIN_ID: u64 = 1_001;
pub const LOCK_ID_BUTTON: u64 = 1_007;

pub fn utxo_recover(config_path: &Path, height: u64) {
    let storage_config = StorageConfig::new(config_path.to_str().unwrap());
    let db = DB::new(&storage_config.db_path, &storage_config);
    let controller_config = ControllerConfig::new(config_path.to_str().unwrap());

    for lock_id in LOCK_ID_VERSION..LOCK_ID_BUTTON {
        match db.load(0, lock_id.to_be_bytes().to_vec()) {
            Ok(data_or_tx_hash) => {
                if data_or_tx_hash.len() == controller_config.hash_len as usize
                    && lock_id != LOCK_ID_CHAIN_ID
                {
                    handle_utxo_tx(&db, data_or_tx_hash, height, lock_id, false);
                } else {
                    println!("lock_id({}) never change from genesis", lock_id);
                }
            }
            Err(status) => {
                println!("load utxo({}) met error: {:?}. Is this a new chain or version lower than v6.3.2", lock_id, status);
            }
        }
    }
}

pub fn handle_utxo_tx(db: &DB, tx_hash: Vec<u8>, height: u64, lock_id: u64, modify: bool) {
    let height_bytes = db.load(7, tx_hash.clone()).unwrap();
    let mut buf: [u8; 8] = [0; 8];
    buf.clone_from_slice(&height_bytes[..8]);
    let tx_hight = u64::from_be_bytes(buf);

    if tx_hight > height {
        match db.load(1, tx_hash) {
            Ok(raw_tx_bytes) => {
                if let UtxoTx(tx) = RawTransaction::decode(raw_tx_bytes.as_slice())
                    .unwrap()
                    .tx
                    .unwrap()
                {
                    let pre_tx_hash = tx.transaction.unwrap().pre_tx_hash;
                    if pre_tx_hash == vec![0u8; 33] {
                        println!("delete lock_id({}) content to be init state", lock_id);
                        db.delete(0, lock_id.to_be_bytes().to_vec()).unwrap();
                    } else {
                        handle_utxo_tx(db, pre_tx_hash, height, lock_id, true);
                    }
                } else {
                    panic!("lock_id({}) tx is not utxo", lock_id);
                }
            }
            Err(status) => {
                panic!("load tx stored at lock_id: {} failed: {}", lock_id, status)
            }
        }
    } else if modify {
        println!(
            "modify lock_id({}) with tx_hash(0x{})",
            lock_id,
            hex::encode(&tx_hash)
        );
        db.store(0, lock_id.to_be_bytes().to_vec(), tx_hash)
            .unwrap();
    } else {
        println!("lock_id({}) keep change", lock_id);
    }
}
