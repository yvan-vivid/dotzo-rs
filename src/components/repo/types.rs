use std::path::{Path, PathBuf};

use derive_more::derive::Constructor;

use crate::{components::environment::types::Environment, config::rc::types::Rc};

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
    pub fn from_config(environment: &Environment, rc: &Rc, given: Option<PathBuf>) -> Self {
        Self::new(given.unwrap_or_else(|| rc.repo.location.to_path(&environment.home)))
    }

    pub fn etc(&self) -> PathBuf {
        self.path.join("etc")
    }
}
