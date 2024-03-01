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

mod cloud_rollback;
mod executor_rollback;
mod rocksdb_rollback;
mod storage_rollback;

use crate::util::{executor_db_path, read_current_height, storage_db, StorageDb};
pub use cloud_rollback::cloud_storage_rollback;
pub use executor_rollback::executor_rollback;
pub use rocksdb_rollback::rocksdb_rollback;
use std::path::Path;
pub use storage_rollback::storage_rollback;

pub async fn rollback(config_path: &Path, height: u64, clean_consensus_data: bool) {
    let storage_db = storage_db(config_path).await;

    let current_height = read_current_height(&storage_db).await;
    println!("current height: {}", current_height);
    println!("rollback height: {}", height);
    if height >= current_height {
        panic!(
            "rollback height({}) >= current height({})",
            height, current_height
        );
    }

    // rollback storage
    if let StorageDb::Opendal(storager) = &storage_db {
        storage_rollback(storager, height, clean_consensus_data).await;
    } else if let StorageDb::RocksDB(storager) = &storage_db {
        rocksdb_rollback(storager, height, true, clean_consensus_data);
    } else {
        panic!("not support storage type");
    }

    // rollback executor
    let executor_db_path = &executor_db_path(config_path);
    executor_rollback(executor_db_path, height);
}
