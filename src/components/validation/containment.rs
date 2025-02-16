use derive_more::Constructor;
use log::{error, info};
use std::path::Path;
use thiserror::Error;

use crate::util::fs::{LinkReader, MetadataChecks};

#[derive(Debug, Error)]
pub enum ContainmentError {
    #[error("Io error")]
    Io(#[from] std::io::Error),

    #[error("The conainer is not a directory")]
    ContainerNotADirectory,

    #[error("The path is not in container")]
    NotContained,
}

pub type Result<T> = core::result::Result<T, ContainmentError>;

#[derive(Debug, Constructor)]
pub struct ContainmentCheck<'a, MC: MetadataChecks, LR: LinkReader> {
    metadata_checks: &'a MC,
    link_reader: &'a LR,
}

impl<MC: MetadataChecks, LR: LinkReader> ContainmentCheck<'_, MC, LR> {
    pub fn check<P: AsRef<Path>, Q: AsRef<Path>>(&self, path: P, container: Q) -> Result<()> {
        let path = self.link_reader.canonicalize(path.as_ref())?;
        let container = self.link_reader.canonicalize(container.as_ref())?;

        info!("Checking {} is in {}", path.display(), container.display());
        if !self.metadata_checks.is_dir(&container) {
            error!("Container is not a directory: {}", path.display());
            return Err(ContainmentError::ContainerNotADirectory);
        }

        if !path.starts_with(&container) {
            error!("{} not not in {}", path.display(), container.display());
            return Err(ContainmentError::NotContained);
        }

        Ok(())
    }
}
