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

extern crate core;

use crate::backup::state_backup;
use crate::recover::{recover, state_recover};
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
    /// hot backup executor_evm & chain db, ONLY USE IN EVM MODE
    #[clap(arg_required_else_help = true)]
    StateBackup {
        /// chain config path
        #[clap(parse(from_os_str), short, long, default_value = "config.toml")]
        config_path: PathBuf,
        /// node root path
        #[clap(parse(from_os_str), short, long, default_value = ".")]
        node_root: PathBuf,
        /// backup path dir
        #[clap(parse(from_os_str), short, long, default_value = "backup/state")]
        backup_path: PathBuf,
        #[clap(required = true)]
        height: u64,
        /// choice crypto server, sm or eth
        #[clap(long, default_value = "sm")]
        crypto: String,
        /// choice consensus server, bft or raft
        #[clap(long, default_value = "bft")]
        consensus: String,
    },
    /// recover chain from early state, ONLY USE IN EVM MODE
    #[clap(arg_required_else_help = true)]
    StateRecover {
        /// chain config path
        #[clap(parse(from_os_str), short, long, default_value = "config.toml")]
        config_path: PathBuf,
        /// node root path
        #[clap(parse(from_os_str), short, long, default_value = ".")]
        node_root: PathBuf,
        /// backup path dir
        #[clap(parse(from_os_str), short, long, default_value = "backup/state")]
        backup_path: PathBuf,
        #[clap(required = true)]
        height: u64,
        /// choice crypto server, sm or eth
        #[clap(long, default_value = "sm")]
        crypto: String,
        /// choice consensus server, bft or raft
        #[clap(long, default_value = "bft")]
        consensus: String,
    },
    /// recover chain status to specified height, ONLY USE IN EVM MODE
    #[clap(arg_required_else_help = true)]
    Recover {
        /// chain config path
        #[clap(parse(from_os_str), short, long, default_value = "config.toml")]
        config_path: PathBuf,
        /// node root path
        #[clap(parse(from_os_str), short, long, default_value = ".")]
        node_root: PathBuf,
        /// the specified height that you want to recover to
        #[clap(required = true)]
        height: u64,
        /// choice crypto server, sm or eth
        #[clap(long, default_value = "sm")]
        crypto: String,
        /// choice consensus server, bft or raft
        #[clap(long, default_value = "bft")]
        consensus: String,
    },
}

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Commands::StateBackup {
            mut config_path,
            node_root,
            mut backup_path,
            height,
            crypto,
            consensus,
        } => {
            if !config_path.is_absolute() {
                config_path = current_dir().unwrap().join(config_path);
            }
            if !backup_path.is_absolute() {
                backup_path = current_dir().unwrap().join(backup_path);
            }
            assert!(set_current_dir(&node_root).is_ok());

            state_backup(
                config_path,
                backup_path,
                height,
                consensus.as_str().into(),
                crypto.as_str().into(),
            );
        }
        Commands::StateRecover {
            mut config_path,
            node_root,
            mut backup_path,
            height,
            crypto,
            consensus,
        } => {
            if !config_path.is_absolute() {
                config_path = current_dir().unwrap().join(config_path);
            }
            if !backup_path.is_absolute() {
                backup_path = current_dir().unwrap().join(backup_path);
            }
            assert!(set_current_dir(&node_root).is_ok());

            state_recover(
                config_path,
                backup_path,
                height,
                consensus.as_str().into(),
                crypto.as_str().into(),
            );
        }
        Commands::Recover {
            mut config_path,
            node_root,
            height,
            crypto,
            consensus,
        } => {
            if !config_path.is_absolute() {
                config_path = current_dir().unwrap().join(config_path);
            }
            assert!(set_current_dir(&node_root).is_ok());

            recover(
                config_path,
                height,
                consensus.as_str().into(),
                crypto.as_str().into(),
            );
        }
    }
}

mod backup;
mod config;
mod crypto;
mod recover;
mod storage;
