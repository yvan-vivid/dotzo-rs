use derive_more::derive::{Constructor, From};
use log::{error, info};
use std::path::{Path, PathBuf};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum HomeError {
    #[error("Given home directory does not exist")]
    DoesNotExist,

    #[error("Given home directory is not a directory")]
    IsNotADirectory,
}

pub type Result<T> = core::result::Result<T, HomeError>;

#[derive(Debug, Constructor, PartialEq, From, Eq)]
pub struct Home(PathBuf);

impl AsRef<Path> for Home {
    fn as_ref(&self) -> &Path {
        self.0.as_path()
    }
}

impl Home {
    pub fn check(&self) -> Result<()> {
        if self.0.is_dir() {
            info!("Home confirmed at: {}", self.0.display());
            Ok(())
        } else if self.0.exists() {
            error!("Given home is not a directory: {}", self.0.display());
            Err(HomeError::IsNotADirectory)
        } else {
            error!("Given home does not exist: {}", self.0.display());
            Err(HomeError::DoesNotExist)
        }
    }
}
