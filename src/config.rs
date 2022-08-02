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

use cloud_util::common::read_toml;
use serde_derive::Deserialize;

#[derive(Debug, Deserialize, Clone)]
#[serde(default)]
pub struct BftConsensusConfig {
    pub wal_path: String,
}

impl Default for BftConsensusConfig {
    fn default() -> Self {
        Self {
            wal_path: "./data/wal".to_string(),
        }
    }
}

impl BftConsensusConfig {
    pub fn new(config_str: &str) -> Self {
        read_toml(config_str, "consensus_bft")
    }
}

#[derive(Debug, Deserialize, Clone)]
#[serde(default)]
pub struct RaftConsensusConfig {
    pub wal_path: String,
}

impl Default for RaftConsensusConfig {
    fn default() -> Self {
        Self {
            wal_path: "./data/wal".to_string(),
        }
    }
}

impl RaftConsensusConfig {
    pub fn new(config_str: &str) -> Self {
        read_toml(config_str, "consensus_raft")
    }
}

pub enum ConsensusType {
    Bft,
    Raft,
}

impl From<&str> for ConsensusType {
    fn from(str: &str) -> Self {
        match str {
            "bft" => ConsensusType::Bft,
            "Bft" => ConsensusType::Bft,
            "BFT" => ConsensusType::Bft,
            "raft" => ConsensusType::Raft,
            "Raft" => ConsensusType::Raft,
            "RAFT" => ConsensusType::Raft,
            _ => panic!("consensus type only bft or raft"),
        }
    }
}

#[derive(Debug, Deserialize, Clone)]
#[serde(default)]
pub struct ControllerConfig {
    pub wal_path: String,
    pub hash_len: u32,
}

impl Default for ControllerConfig {
    fn default() -> Self {
        Self {
            wal_path: "./data/wal".to_string(),
            hash_len: 32,
        }
    }
}

impl ControllerConfig {
    pub fn new(config_str: &str) -> Self {
        read_toml(config_str, "controller")
    }
}

#[derive(Debug, Deserialize, Clone)]
#[serde(default)]
pub struct ExecutorConfig {
    pub db_path: String,
}

impl Default for ExecutorConfig {
    fn default() -> Self {
        Self {
            db_path: "data".to_string(),
        }
    }
}

impl ExecutorConfig {
    pub fn new(config_str: &str) -> Self {
        read_toml(config_str, "executor_evm")
    }
}
