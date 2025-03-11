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
    fn make_dir(&self, path: impl AsRef<Path>) -> Result<()>;
    fn symlink(&self, target: impl AsRef<Path>, path: impl AsRef<Path>) -> Result<()>;
}

#[derive(Debug, Constructor)]
pub struct StandardFsActions {}

impl ActionError {
    pub fn from_io_kind(kind: ErrorKind) -> Self {
        std::io::Error::from(kind).into()
    }
}

impl Actions for StandardFsActions {
    fn make_dir(&self, path: impl AsRef<Path>) -> Result<()> {
        let path = path.as_ref();
        info!("Creating directory: {}", path.display());
        create_dir_all(path)?;
        Ok(())
    }

    fn symlink(&self, target: impl AsRef<Path>, path: impl AsRef<Path>) -> Result<()> {
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

#[cfg(test)]
mod testing {
    use std::cell::RefCell;

    use crate::util::fs::testing::{TestFile, TestFs};

    use super::*;

    #[derive(Debug, Constructor)]
    pub struct TestActions {
        pub fs: RefCell<TestFs>,
    }

    impl Actions for TestActions {
        fn make_dir(&self, path: impl AsRef<Path>) -> Result<()> {
            self.fs.borrow_mut().add_directory(path);
            Ok(())
        }

        fn symlink(&self, target: impl AsRef<Path>, path: impl AsRef<Path>) -> Result<()> {
            let mut fs = self.fs.borrow_mut();
            let target = target.as_ref();
            let path = path.as_ref();

            if fs.exists(target) {
                Err(std::io::Error::from(ErrorKind::AlreadyExists).into())
            } else {
                fs.add_file(target.to_owned(), TestFile::Symlink(path.to_owned()));
                Ok(())
            }
        }
    }
}
