use log::{error, info};
use thiserror::Error;

use crate::app::{
    cli::{Cli, Command},
    types::App,
};

use super::{
    info::{info_task, InfoTaskError},
    init::{init_task, InitTaskError},
    sync::{sync_task, SyncTaskError},
};

#[derive(Debug, Error)]
pub enum RunTaskError {
    #[error("Problem with initialization")]
    Init(#[from] InitTaskError),

    #[error("Problem with the environment")]
    Sync(#[from] SyncTaskError),

    #[error("Problem with the environment")]
    Info(#[from] InfoTaskError),
}

pub type Result<T> = core::result::Result<T, RunTaskError>;

pub fn run<'a, APP: App<'a>>(app: &'a APP, cli: &Cli) -> Result<()> {
    info!("Initializing Dotzo");
    let dotzo = init_task(app, cli)?;

    info!("Running task: {:?}", cli.command);
    match cli.command {
        Command::Init => Ok(()),
        Command::Sync => Ok(sync_task(app, cli, dotzo)?),
        Command::Info => Ok(info_task(dotzo.environment)?),
    }
}
