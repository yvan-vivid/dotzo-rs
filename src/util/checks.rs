use derive_more::Constructor;
use inquire::{Confirm, InquireError};
use log::{error, info};
use std::path::Path;
use thiserror::Error;

use super::{
    actions::{ActionError, Actions},
    fs::MetadataChecks,
};

#[derive(Debug, Error)]
pub enum DirectoryCheckError {
    #[error("Given directory does not exist")]
    DoesNotExist,

    #[error("Given directory is not a directory")]
    IsNotADirectory,

    #[error("Inquire error")]
    InquireError(#[from] InquireError),

    #[error("File system action error")]
    ActionError(#[from] ActionError),
}

pub type Result<T> = core::result::Result<T, DirectoryCheckError>;

#[derive(Debug, Constructor)]
pub struct DirectoryCheck<'a, MC: MetadataChecks, A: Actions> {
    metadata_checks: &'a MC,
    actions: &'a A,
}

impl<MC: MetadataChecks, A: Actions> DirectoryCheck<'_, MC, A> {
    pub fn create<P: AsRef<Path>>(&self, path: P) -> Result<bool> {
        let path = path.as_ref();

        // TODO: Inject
        let do_create_config = Confirm::new(&format!("Create config at {}", path.display()))
            .with_default(false)
            .prompt()?;

        if do_create_config {
            info!("Creating config at {}", path.display());
            self.actions.make_dir(path)?;
            info!("Config created");
            Ok(true)
        } else {
            info!("Not creating config directory");
            Ok(false)
        }
    }

    pub fn check<P: AsRef<Path>>(&self, path: P, should_create: bool) -> Result<()> {
        let path = path.as_ref();
        if self.metadata_checks.is_dir(path) {
            info!("Confirmed at: {}", path.display());
            Ok(())
        } else if self.metadata_checks.exists(path) {
            error!("Is not a directory: {}", path.display());
            Err(DirectoryCheckError::IsNotADirectory)
        } else if should_create {
            if self.create(path)? {
                Ok(())
            } else {
                Err(DirectoryCheckError::DoesNotExist)
            }
        } else {
            error!("Does not exist: {}", path.display());
            Err(DirectoryCheckError::DoesNotExist)
        }
    }
}
