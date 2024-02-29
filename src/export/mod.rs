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
use crate::util::{
    copy_dir, executor_db_path, get_real_key, read_current_height, storage_db, StorageDb,
};
use cita_cloud_proto::blockchain::raw_transaction::Tx::UtxoTx;
use cita_cloud_proto::blockchain::Block;
use cita_cloud_proto::storage::Regions;
use std::path::{Path, PathBuf};
use storage_opendal::{config::StorageConfig as OpendalConfig, storager::Storager};

use prost::Message;

pub async fn export(
    config_path: PathBuf,
    export_path: PathBuf,
    begin_height: u64,
    end_height: u64,
) {
    println!("export height: [{}, {}]", begin_height, end_height);
    let storage_db = storage_db(&config_path).await;
    let executor_db_path = executor_db_path(&config_path);

    let current_height = read_current_height(&storage_db).await;
    println!("current height: {}", current_height);

    if end_height > current_height || begin_height > end_height {
        panic!(
            "invalid height! current height: {}, begin height: {}, end height: {}",
            current_height, begin_height, end_height
        );
    }

    // export executor state
    let state_path = executor_db_path.to_owned() + "/statedb";
    let state_export_path = export_path.clone().join("data/statedb");

    state_snapshot_backup(&state_path, &state_export_path, end_height);
    println!("export excutor state done!");

    // backup executor chain_db
    let chain_path = executor_db_path.to_owned() + "/nosql";
    let executor_export_path = export_path.clone().join("data/nosql");
    copy_dir(Path::new(&chain_path), &executor_export_path);
    println!("copy excutor chain_db done!");

    // export storage data
    // convert storage rocksdb data to storage opendal data
    if let StorageDb::RocksDB(read) = &storage_db {
        let storage_export_path = export_path.clone().join("chain_data");
        let config = OpendalConfig::default();
        let write = Storager::build(
            storage_export_path.to_str().unwrap(),
            &config.cloud_storage,
            config.l1_capacity,
            config.l2_capacity,
            u32::MAX as u64,
            config.retreat_interval,
        )
        .await;
        for height in begin_height..=end_height {
            print!("\rconverting old: {}", height);
            let height_bytes = height.to_be_bytes().to_vec();
            let old_block_bytes = read.load_full_block(height_bytes.clone()).unwrap();
            let mut old_bytes = read.load(4, height_bytes.clone()).unwrap();
            old_bytes.extend_from_slice(&old_block_bytes);

            write
                .store_all_block_data(&height_bytes, &old_bytes)
                .await
                .unwrap();

            // handle utxo tx
            let block = Block::decode(old_block_bytes.as_slice()).unwrap();
            let global_region = i32::from(Regions::Global) as u32;
            for raw_tx in block.body.unwrap().body {
                if let UtxoTx(utxo_tx) = raw_tx.tx.unwrap() {
                    let tx_hash = utxo_tx.transaction_hash;
                    let lock_id = utxo_tx.transaction.unwrap().lock_id;

                    write
                        .store(
                            &get_real_key(global_region, lock_id.to_be_bytes().as_ref()),
                            tx_hash.as_slice(),
                        )
                        .await
                        .unwrap();
                }
            }
        }
        println!("\nexport block done!");
    } else if let StorageDb::Opendal(read) = &storage_db {
        // export storage opendal data to storage opendal data
        let storage_export_path = export_path.clone().join("chain_data");
        let config = OpendalConfig::default();
        let write = Storager::build(
            storage_export_path.to_str().unwrap(),
            &config.cloud_storage,
            config.l1_capacity,
            config.l2_capacity,
            u32::MAX as u64,
            config.retreat_interval,
        )
        .await;
        for height in begin_height..=end_height {
            print!("\rexporting: {}", height);
            let height_bytes = height.to_be_bytes().to_vec();
            let block_bytes = read.load_full_block(&height_bytes).await.unwrap();
            let mut block_hash_bytes = read
                .load(&get_real_key(4, &height_bytes), true)
                .await
                .unwrap();
            block_hash_bytes.extend_from_slice(&block_bytes);

            write
                .store_all_block_data(&height_bytes, &block_hash_bytes)
                .await
                .unwrap();

            // handle utxo tx
            let block = Block::decode(block_bytes.as_slice()).unwrap();
            let global_region = i32::from(Regions::Global) as u32;
            for raw_tx in block.body.unwrap().body {
                if let UtxoTx(utxo_tx) = raw_tx.tx.unwrap() {
                    let tx_hash = utxo_tx.transaction_hash;
                    let lock_id = utxo_tx.transaction.unwrap().lock_id;

                    write
                        .store(
                            &get_real_key(global_region, lock_id.to_be_bytes().as_ref()),
                            tx_hash.as_slice(),
                        )
                        .await
                        .unwrap();
                }
            }
        }
        println!("\nexport block done!");
    } else {
        panic!("storage db not found")
    }

    println!("export done!");
}
