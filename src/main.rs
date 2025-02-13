mod app;
mod components;
mod config;
mod dotzo;
mod mapping;
mod tasks;
mod util;

use std::path::PathBuf;

use anyhow::Result;
use components::environment::home::Home;

use app::{
    cli::{parse_cli, Command},
    logging::setup_logging,
};
use dotzo::Dotzo;
use tasks::{info::info_task, sync::sync_task};
use util::fs::StandardFs;

fn main() -> Result<()> {
    let cli = parse_cli();
    setup_logging()?;

    let standard_fs = StandardFs::new();
    let home: Home = PathBuf::from("/home/hexxiiiz").into();
    let dotzo = Dotzo::from_config_path(home)?;

    // TODO: SyncSecure => Make links into <home>/.ssh from secure repo
    match cli.command {
        Command::Init => unimplemented!(),
        Command::Setup => unimplemented!(),
        Command::Sync => sync_task(dotzo, standard_fs),
        Command::Info => info_task(dotzo),
    }
}
