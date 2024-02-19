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
use cita_cloud_proto::blockchain::raw_transaction::Tx::UtxoTx;
use cita_cloud_proto::blockchain::Block;
use cita_cloud_proto::storage::Regions;
use std::path::{Path, PathBuf};
use storage_opendal::{config::StorageConfig as OpendalConfig, storager::Storager};
use storage_rocksdb::config::StorageConfig as RocksdbConfig;

use std::fs;
use toml::Table;

use prost::Message;

pub async fn backup(
    config_path: PathBuf,
    mut backup_path: PathBuf,
    height: Option<u64>,
    export_data: bool,
) {
    // check height
    let storage_db = storage_db(&config_path).await;
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
    if export_data {
        state_snapshot_backup(&state_path, &state_backup_path, backup_height);
        println!("export excutor state done!");
    } else {
        copy_dir(Path::new(&state_path), &state_backup_path);
        println!("copy excutor state done!");
    }

    // backup executor chain_db
    let chain_path = executor_db_path.to_owned() + "/nosql";
    let executor_backup_path = backup_path.clone().join("data/nosql");
    copy_dir(Path::new(&chain_path), &executor_backup_path);
    println!("copy excutor chain_db done!");

    // backup storage data
    if export_data {
        // convert storage rocksdb data to storage opendal data
        if let StorageDb::RocksDB(read) = &storage_db {
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
            for height in 0..=backup_height {
                print!("\rconverting old: {}/{}", height, backup_height);
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
            for height in 0..=backup_height {
                print!("\rexporting: {}/{}", height, backup_height);
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
    } else {
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
        executor_rollback(
            backup_path
                .clone()
                .join("data")
                .as_os_str()
                .to_str()
                .unwrap(),
            backup_height,
        );
        storage_rollback(&write, backup_height, false).await;
    }

    println!("backup done!");
}
