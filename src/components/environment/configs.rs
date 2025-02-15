use derive_more::derive::{Constructor, From};
use inquire::{Confirm, InquireError};
use log::{error, info};
use std::path::{Path, PathBuf};
use thiserror::Error;

use crate::components::actions::make_dir;

#[derive(Debug, Error)]
pub enum ConfigsError {
    #[error("Given configs directory does not exist")]
    DoesNotExist,

    #[error("Given configs directory is not a directory")]
    IsNotADirectory,

    #[error("Prompting error")]
    Prompting(#[from] InquireError),

    #[error("IO error")]
    Io(#[from] std::io::Error),
}

pub type Result<T> = core::result::Result<T, ConfigsError>;

#[derive(Debug, Constructor, PartialEq, From, Eq)]
pub struct Configs(PathBuf);

impl AsRef<Path> for Configs {
    fn as_ref(&self) -> &Path {
        self.0.as_path()
    }
}

impl Configs {
    pub fn create_config(&self) -> Result<()> {
        let do_create_config = Confirm::new(&format!("Create config at {}", self.0.display()))
            .with_default(false)
            .prompt()?;

        if do_create_config {
            info!("Creating config at {}", self.0.display());
            make_dir(self.as_ref())?;
            info!("Config created");
            Ok(())
        } else {
            info!("Not creating config directory");
            Err(ConfigsError::DoesNotExist)
        }
    }

    pub fn check(&self) -> Result<()> {
        if self.0.is_dir() {
            info!("Config confirmed at: {}", self.0.display());
            Ok(())
        } else if self.0.exists() {
            error!("Given config is not a directory: {}", self.0.display());
            Err(ConfigsError::IsNotADirectory)
        } else {
            info!("Given config does not exist: {}", self.0.display());
            self.create_config()
        }
    }
}
