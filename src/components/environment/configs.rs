use derive_more::derive::{Constructor, From};
use std::path::{Path, PathBuf};

#[derive(Debug, Constructor, PartialEq, From, Eq)]
pub struct Configs(PathBuf);

impl AsRef<Path> for Configs {
    fn as_ref(&self) -> &Path {
        self.0.as_path()
    }
}
