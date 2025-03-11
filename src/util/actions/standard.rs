use derive_more::derive::Constructor;
use std::{fs::create_dir_all, os::unix::fs::symlink, path::Path};

use log::info;

use super::types::{Actions, Result};

#[derive(Debug, Constructor)]
pub struct StandardActions {}

impl Actions for StandardActions {
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
