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

mod backup;
mod rollback;
mod util;

use crate::backup::backup;
use crate::rollback::{cloud_storage_rollback, rollback};
use clap::{Parser, Subcommand};
use std::env::{current_dir, set_current_dir};
use std::path::PathBuf;

/// Simple program to greet a person
#[derive(Parser)]
#[clap(author, version, about, long_about = None)]
struct Cli {
    #[clap(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// rollback chain status to specified height, ONLY USE IN EVM MODE
    #[clap(arg_required_else_help = true)]
    Rollback {
        /// chain config path
        #[clap(short, long, default_value = "config.toml")]
        config_path: PathBuf,
        /// node root path
        #[clap(short, long, default_value = ".")]
        node_root: PathBuf,
        /// the specified height that you want to rollback to
        #[clap(required = true)]
        height: u64,
        /// whether to clean consensus data
        #[clap(long = "clean")]
        clean_consensus_data: bool,
    },
    /// rollback cloud storage status to specified height, ONLY USE IN EVM MODE
    #[clap(arg_required_else_help = true)]
    CloudRollback {
        /// chain config path
        #[clap(short, long, default_value = "config.toml")]
        config_path: PathBuf,
        /// node root path
        #[clap(short, long, default_value = ".")]
        node_root: PathBuf,
        /// the specified height that you want to rollback to
        #[clap(required = true)]
        height: u64,
    },
    /// backup executor and storage data of a specified height
    Backup {
        /// chain config path
        #[clap(short, long, default_value = "config.toml")]
        config_path: PathBuf,
        /// node root path
        #[clap(short, long, default_value = ".")]
        node_root: PathBuf,
        /// backup path dir
        #[clap(short, long, default_value = "backup")]
        backup_path: PathBuf,
        /// backup height
        #[clap(short, long)]
        height: Option<u64>,
        /// whether to only backup executor state
        #[clap(short, long)]
        snapshot_only: bool,
        /// whether storage is rocksdb, if so, data will be converted
        #[clap(short, long)]
        old_storage: bool,
    },
}
#[tokio::main]
async fn main() {
    let command = Cli::parse().command;
    operate(command).await;
}

async fn operate(command: Commands) {
    match command {
        Commands::Rollback {
            mut config_path,
            node_root,
            height,
            clean_consensus_data,
        } => {
            if !config_path.is_absolute() {
                config_path = current_dir().unwrap().join(config_path);
            }
            assert!(set_current_dir(&node_root).is_ok());

            rollback(&config_path, height, clean_consensus_data).await;
        }
        Commands::CloudRollback {
            mut config_path,
            node_root,
            height,
        } => {
            if !config_path.is_absolute() {
                config_path = current_dir().unwrap().join(config_path);
            }
            assert!(set_current_dir(&node_root).is_ok());

            cloud_storage_rollback(&config_path, height).await;
        }
        Commands::Backup {
            mut config_path,
            node_root,
            mut backup_path,
            height,
            snapshot_only,
            old_storage,
        } => {
            if !config_path.is_absolute() {
                config_path = current_dir().unwrap().join(config_path);
            }
            assert!(set_current_dir(node_root).is_ok());

            if !backup_path.is_absolute() {
                backup_path = current_dir().unwrap().join(backup_path);
            }

            backup(config_path, backup_path, height, snapshot_only, old_storage).await;
        }
    }
}
