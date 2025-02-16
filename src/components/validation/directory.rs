use derive_more::Constructor;
use log::{error, info};
use std::path::Path;
use thiserror::Error;

use crate::util::fs::MetadataChecks;

#[derive(Debug, Error)]
pub enum DirectoryCheckError {
    #[error("Given directory does not exist")]
    DoesNotExist,

    #[error("Given directory is not a directory")]
    IsNotADirectory,
}

pub type Result<T> = core::result::Result<T, DirectoryCheckError>;

#[derive(Debug, Constructor)]
pub struct DirectoryCheck<'a, MC: MetadataChecks> {
    metadata_checks: &'a MC,
}

impl<MC: MetadataChecks> DirectoryCheck<'_, MC> {
    //pub fn create<P: AsRef<Path>>(&self, path: P) -> Result<bool> {
    //    let path = path.as_ref();
    //    let do_create_config = self
    //        .prompter
    //        .confirm(format!("Create config at {}", path.display()), false)
    //        .map_err(|e| DirectoryCheckError::PromptError(Box::new(e)))?;
    //
    //    if do_create_config {
    //        info!("Creating config at {}", path.display());
    //        self.actions.make_dir(path)?;
    //        info!("Config created");
    //        Ok(true)
    //    } else {
    //        info!("Not creating config directory");
    //        Ok(false)
    //    }
    //}

    pub fn check<P: AsRef<Path>>(&self, path: P) -> Result<()> {
        let path = path.as_ref();
        if self.metadata_checks.is_dir(path) {
            info!("Confirmed at: {}", path.display());
            Ok(())
        } else if self.metadata_checks.exists(path) {
            error!("Is not a directory: {}", path.display());
            Err(DirectoryCheckError::IsNotADirectory)
        } else {
            error!("Does not exist: {}", path.display());
            Err(DirectoryCheckError::DoesNotExist)
        }
    }
}
