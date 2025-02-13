use std::path::{Path, PathBuf};

use derive_more::derive::Constructor;
use relative_path::RelativePathBuf;

use crate::components::environment::home::Home;

#[derive(Debug, Constructor, PartialEq, Eq)]
pub struct Repo {
    pub path: PathBuf,
}

impl AsRef<Path> for Repo {
    fn as_ref(&self) -> &Path {
        &self.path
    }
}

impl Repo {
    pub fn from_home(home: &Home) -> Self {
        Self {
            path: RelativePathBuf::from("_").to_path(home),
        }
    }

    pub fn etc(&self) -> PathBuf {
        self.path.join("etc")
    }
}
