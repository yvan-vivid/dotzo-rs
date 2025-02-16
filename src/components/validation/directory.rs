use derive_more::Constructor;
use log::{error, info};
use std::path::Path;
use thiserror::Error;

use crate::util::{
    actions::{ActionError, Actions},
    fs::MetadataChecks,
    prompting::Prompter,
};

#[derive(Debug, Error)]
pub enum DirectoryCheckError<E: std::error::Error> {
    #[error("Given directory does not exist")]
    DoesNotExist,

    #[error("Given directory is not a directory")]
    IsNotADirectory,

    #[error("Inquire error")]
    PromptError(E),

    #[error("File system action error")]
    ActionError(#[from] ActionError),
}

pub type Result<T, E> = core::result::Result<T, DirectoryCheckError<E>>;

#[derive(Debug, Constructor)]
pub struct DirectoryCheck<'a, MC: MetadataChecks, A: Actions, PR: Prompter> {
    metadata_checks: &'a MC,
    actions: &'a A,
    prompter: &'a PR,
}

impl<MC: MetadataChecks, A: Actions, PR: Prompter> DirectoryCheck<'_, MC, A, PR> {
    pub fn create<P: AsRef<Path>>(&self, path: P) -> Result<bool, PR::Error> {
        let path = path.as_ref();
        let do_create_config = self
            .prompter
            .confirm(format!("Create config at {}", path.display()), false)
            .map_err(DirectoryCheckError::PromptError)?;

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

    pub fn check<P: AsRef<Path>>(&self, path: P, should_create: bool) -> Result<(), PR::Error> {
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
