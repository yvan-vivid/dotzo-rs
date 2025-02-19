use std::path::PathBuf;

use clap::{Parser, Subcommand};
use clap_verbosity_flag::{Verbosity, WarnLevel};

/// Yvan Vivid's tool to manage his (or your) home environment  
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Command,

    /// Override for home directory
    #[arg(long)]
    pub home_dir: Option<PathBuf>,

    /// Override for config directory
    #[arg(long)]
    pub config_dir: Option<PathBuf>,

    /// Override for repo location
    #[arg(long)]
    pub repo: Option<PathBuf>,

    /// Configuration file path (dotrc file)
    #[arg(long)]
    pub config: Option<PathBuf>,

    /// Dry run
    #[arg(long)]
    pub dry_run: bool,

    /// Level of verbosity - defaults to warn, -v for info, -vv for debug
    #[command(flatten)]
    pub verbose: Verbosity<WarnLevel>,
}

#[derive(Subcommand, Debug)]
pub enum Command {
    // Init,
    // Setup,
    /// Sync dotfiles from repo to home environment
    Sync,

    /// Show info about home environment
    Info,
}

pub fn parse_cli() -> Cli {
    Cli::parse()
}
