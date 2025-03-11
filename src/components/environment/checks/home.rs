use derive_more::derive::Constructor;
use log::error;
use std::path::PathBuf;
use thiserror::Error;

use crate::{
    components::environment::types::Home,
    util::fs::MetadataChecks,
    validation::directory::{DirectoryCheck, DirectoryCheckError},
};

#[derive(Debug, Error)]
pub enum HomeCheckError {
    #[error("Home check failed at {path} with {error:?}")]
    Check {
        path: PathBuf,
        error: DirectoryCheckError,
    },
}

pub type Result<T> = std::result::Result<T, HomeCheckError>;

#[derive(Debug, Constructor)]
pub struct HomeCheck<'a, MC: MetadataChecks> {
    pub directory_check: DirectoryCheck<'a, MC>,
}

impl<MC: MetadataChecks> HomeCheck<'_, MC> {
    pub fn check(&self, home: &Home) -> Result<()> {
        self.directory_check
            .check(home)
            .map_err(|error| HomeCheckError::Check {
                path: home.as_ref().to_owned(),
                error,
            })
    }
}
