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

use crate::util::{get_real_key, read_current_height, storage_db, StorageDb};
use std::path::Path;

pub async fn set_height(config_path: &Path, height: u64) {
    let storage_db = storage_db(config_path).await;

    let current_height = read_current_height(&storage_db).await;
    println!("current height: {}", current_height);
    println!("set height: {}", height);

    if let StorageDb::Opendal(storager) = &storage_db {
        storager
            .store(&get_real_key(0, &0u64.to_be_bytes()), &height.to_be_bytes())
            .await
            .unwrap();
    } else {
        panic!("not support storage type");
    }
}
