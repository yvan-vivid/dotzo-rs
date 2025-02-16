use derive_more::derive::Constructor;
use std::{fs::create_dir_all, io::ErrorKind, os::unix::fs::symlink, path::Path};
use thiserror::Error;

use log::info;

use super::fs::MetadataChecks;

#[derive(Debug, Error)]
pub enum ActionError {
    #[error("IO error")]
    Io(#[from] std::io::Error),
}

pub type Result<T> = core::result::Result<T, ActionError>;

pub trait Actions {
    fn make_dir<P: AsRef<Path>>(&self, path: P) -> Result<()>;
    fn symlink<P: AsRef<Path>, Q: AsRef<Path>>(&self, target: P, path: Q) -> Result<()>;

    fn try_symlink<P: AsRef<Path>, Q: AsRef<Path>>(&self, target: P, path: Q) -> Result<bool> {
        match self.symlink(target, path) {
            Ok(_) => Ok(true),
            Err(e) => match e {
                ActionError::Io(ioe) => match ioe.kind() {
                    ErrorKind::AlreadyExists => Ok(false),
                    _ => Err(ioe.into()),
                },
            },
        }
    }
}

#[derive(Debug, Constructor)]
pub struct StandardFsActions {}

impl Actions for StandardFsActions {
    fn make_dir<P: AsRef<Path>>(&self, path: P) -> Result<()> {
        let path = path.as_ref();
        info!("Creating directory: {}", path.display());
        create_dir_all(path)?;
        Ok(())
    }

    fn symlink<P: AsRef<Path>, Q: AsRef<Path>>(&self, target: P, path: Q) -> Result<()> {
        info!(
            "Creating symlink from {} to {}",
            target.as_ref().display(),
            path.as_ref().display()
        );
        Ok(symlink(path, target)?)
    }
}

#[derive(Debug, Constructor)]
pub struct DryFsActions<'a, MC: MetadataChecks> {
    metadata_checks: &'a MC,
}

impl<MC: MetadataChecks> Actions for DryFsActions<'_, MC> {
    fn make_dir<P: AsRef<Path>>(&self, path: P) -> Result<()> {
        let path = path.as_ref();
        info!("DRY-RUN: Would have created directory: {}", path.display());
        Ok(())
    }

    fn symlink<P: AsRef<Path>, Q: AsRef<Path>>(&self, target: P, path: Q) -> Result<()> {
        if self.metadata_checks.exists(&target) {
            Err(std::io::Error::new(ErrorKind::AlreadyExists, "Target already exists"))?
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
