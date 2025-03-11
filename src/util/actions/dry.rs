use derive_more::derive::Constructor;
use std::{io::ErrorKind, path::Path};

use log::info;

use crate::util::fs::MetadataChecks;

use super::types::{Actions, Result};

#[derive(Debug, Constructor)]
pub struct DryActions<'a, MC: MetadataChecks> {
    metadata_checks: &'a MC,
}

impl<MC: MetadataChecks> Actions for DryActions<'_, MC> {
    fn make_dir(&self, path: impl AsRef<Path>) -> Result<()> {
        let path = path.as_ref();
        info!("DRY-RUN: Would have created directory: {}", path.display());
        Ok(())
    }

    fn symlink(&self, target: impl AsRef<Path>, path: impl AsRef<Path>) -> Result<()> {
        if self.metadata_checks.exists(&target) {
            Err(std::io::Error::new(
                ErrorKind::AlreadyExists,
                "Target already exists",
            ))?
        } else {
            info!(
                "DRY-RUN: Would have creating symlink from {} to {}",
                target.as_ref().display(),
                path.as_ref().display()
            );
            Ok(())
        }
    }
}
