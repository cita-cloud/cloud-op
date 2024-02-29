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

use crate::util::{CONTROLLER_WAL, HASH_LEN, OVERLORD_DATA, RAFT_DATA};
use cita_cloud_proto::blockchain::raw_transaction::Tx::UtxoTx;
use cita_cloud_proto::blockchain::{CompactBlock, RawTransaction};
use prost::Message;
use std::fs::remove_dir_all;
use storage_rocksdb::db::DB;

pub fn rocksdb_rollback(
    db: &DB,
    height: u64,
    clean_controller_wal: bool,
    clean_consensus_data: bool,
) {
    rocksdb_utxo_rollback(db, height);
    rocksdb_chain_rollback(db, height, clean_controller_wal, clean_consensus_data);
    println!("storage rollback done!");
}

const LOCK_ID_VERSION: u64 = 1_000;
const LOCK_ID_CHAIN_ID: u64 = 1_001;
const LOCK_ID_BUTTON: u64 = 1_007;

pub fn rocksdb_utxo_rollback(db: &DB, height: u64) {
    for lock_id in LOCK_ID_VERSION..LOCK_ID_BUTTON {
        match db.load(0, lock_id.to_be_bytes().to_vec()) {
            Ok(data_or_tx_hash) => {
                if data_or_tx_hash.len() == HASH_LEN as usize && lock_id != LOCK_ID_CHAIN_ID {
                    handle_utxo_tx(db, data_or_tx_hash, height, lock_id, false);
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

fn handle_utxo_tx(db: &DB, tx_hash: Vec<u8>, height: u64, lock_id: u64, modify: bool) {
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

pub fn rocksdb_chain_rollback(
    db: &DB,
    height: u64,
    clean_controller_wal: bool,
    clean_consensus_data: bool,
) {
    // remove consensus wal file
    if clean_consensus_data {
        let _ = remove_dir_all(OVERLORD_DATA);
        let _ = remove_dir_all(RAFT_DATA);
    }

    // remove controller wal file
    if clean_controller_wal {
        let _ = remove_dir_all(CONTROLLER_WAL);
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
