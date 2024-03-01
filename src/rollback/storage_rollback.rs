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

use crate::util::{get_real_key, HASH_LEN, OVERLORD_DATA, RAFT_DATA};
use async_recursion::async_recursion;
use cita_cloud_proto::blockchain::raw_transaction::Tx::UtxoTx;
use cita_cloud_proto::blockchain::RawTransaction;
use prost::Message;
use std::fs::remove_dir_all;
use storage_opendal::storager::Storager;

const LOCK_ID_VERSION: u64 = 1_000;
const LOCK_ID_CHAIN_ID: u64 = 1_001;
const LOCK_ID_BUTTON: u64 = 1_008;

pub async fn storage_rollback(storager: &Storager, height: u64, clean_consensus_data: bool) {
    utxo_rollback(storager, height).await;
    chain_rollback(storager, height, clean_consensus_data).await;
    println!("storage rollback done!");
}

async fn utxo_rollback(storager: &Storager, height: u64) {
    for lock_id in LOCK_ID_VERSION..LOCK_ID_BUTTON {
        match storager
            .load(&get_real_key(0, &lock_id.to_be_bytes()), true)
            .await
        {
            Ok(data_or_tx_hash) => {
                if data_or_tx_hash.len() == HASH_LEN as usize && lock_id != LOCK_ID_CHAIN_ID {
                    handle_utxo_tx(storager, data_or_tx_hash, height, lock_id, false).await;
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

#[async_recursion]
async fn handle_utxo_tx(
    storager: &Storager,
    tx_hash: Vec<u8>,
    height: u64,
    lock_id: u64,
    modify: bool,
) {
    let height_bytes = storager
        .load(&get_real_key(7, &tx_hash), true)
        .await
        .unwrap();
    let mut buf: [u8; 8] = [0; 8];
    buf.clone_from_slice(&height_bytes[..8]);
    let tx_hight = u64::from_be_bytes(buf);

    if tx_hight > height {
        match storager.load(&get_real_key(1, &tx_hash), true).await {
            Ok(raw_tx_bytes) => {
                if let UtxoTx(tx) = RawTransaction::decode(raw_tx_bytes.as_slice())
                    .unwrap()
                    .tx
                    .unwrap()
                {
                    let pre_tx_hash = tx.transaction.unwrap().pre_tx_hash;
                    if pre_tx_hash == vec![0u8; 33] {
                        println!("delete lock_id({}) content to be init state", lock_id);
                        storager
                            .next_storager
                            .as_ref()
                            .unwrap()
                            .operator
                            .delete(&get_real_key(0, &lock_id.to_be_bytes()))
                            .await
                            .unwrap();
                    } else {
                        handle_utxo_tx(storager, pre_tx_hash, height, lock_id, true).await;
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
        storager
            .store(&get_real_key(0, &lock_id.to_be_bytes()), &tx_hash)
            .await
            .unwrap();
    } else {
        println!("lock_id({}) keep change", lock_id);
    }
}

async fn chain_rollback(storager: &Storager, height: u64, clean_consensus_data: bool) {
    // remove consensus wal file
    if clean_consensus_data {
        let _ = remove_dir_all(OVERLORD_DATA);
        let _ = remove_dir_all(RAFT_DATA);
    }

    // rollback current height and delete height
    // for storage_opendal current hash is a virtual key, so we only need to rollback current height
    storager
        .store(&get_real_key(0, &0u64.to_be_bytes()), &height.to_be_bytes())
        .await
        .unwrap();

    let res = storager
        .load(&get_real_key(0, &2u64.to_be_bytes()), true)
        .await;
    if let Ok(delete_height_bytes) = res {
        let mut buf: [u8; 8] = [0; 8];
        buf.clone_from_slice(&delete_height_bytes[..8]);
        let delete_height = u64::from_be_bytes(buf);
        println!("local storage current delete_height: {}", delete_height);
        let new_delete_height = height.min(delete_height);
        println!(
            "local storage delete_height rollback to: {}",
            new_delete_height
        );

        storager
            .store(
                &get_real_key(0, &2u64.to_be_bytes()),
                &new_delete_height.to_be_bytes(),
            )
            .await
            .unwrap();
    }
}
