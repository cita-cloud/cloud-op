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

#[cfg(all(feature = "crypto_sm", feature = "crypto_eth"))]
compile_error!("features `crypto_sm` and `crypto_eth` are mutually exclusive");
#[cfg(all(feature = "raft", feature = "bft"))]
compile_error!("features `raft` and `bft` are mutually exclusive");

use crate::recover::recover;
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
    Backup {
        /// chain config path
        #[clap(
            required = true,
            parse(from_os_str),
            short,
            default_value = "config.toml"
        )]
        config_path: PathBuf,
        /// node root path
        #[clap(required = true, parse(from_os_str), short)]
        node_root: PathBuf,
    },
    /// recover chain status to specified height, ONLY USE IN EVM MODE
    #[clap(arg_required_else_help = true)]
    Recover {
        /// chain config path
        #[clap(
            required = true,
            parse(from_os_str),
            short,
            default_value = "config.toml"
        )]
        config_path: PathBuf,
        /// node root path
        #[clap(required = true, parse(from_os_str), short, default_value = ".")]
        node_root: PathBuf,
        /// the specified height that you want to recover to
        #[clap(required = true)]
        height: u64,
    },
}

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Commands::Backup {
            config_path: mut _config_path,
            node_root,
        } => {
            if !_config_path.is_absolute() {
                _config_path = current_dir().unwrap().join(_config_path);
            }
            assert!(set_current_dir(&node_root).is_ok());
        }
        Commands::Recover {
            mut config_path,
            node_root,
            height,
        } => {
            if !config_path.is_absolute() {
                config_path = current_dir().unwrap().join(config_path);
            }
            assert!(set_current_dir(&node_root).is_ok());

            recover(config_path, height);
        }
    }
}

mod backup;
mod consensus;
mod crypto;
mod recover;
mod storage;
