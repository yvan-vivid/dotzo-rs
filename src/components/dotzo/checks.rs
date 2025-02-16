use derive_more::derive::Constructor;
use log::error;
use thiserror::Error;

use crate::util::fs::{LinkReader, MetadataChecks};

use crate::components::{
    environment::checks::{EnvironmentChecker, EnvironmentCheckerError},
    repo::types::RepoError,
};

use super::types::Dotzo;

#[derive(Debug, Error)]
pub enum DotzoError {
    #[error("Environment Error")]
    Environment(#[from] EnvironmentCheckerError),

    #[error("Environment Error")]
    Repo(#[from] RepoError),
}

pub type Result<T> = std::result::Result<T, DotzoError>;

#[derive(Debug, Constructor)]
pub struct DotzoChecker<'a, MC: MetadataChecks, LR: LinkReader> {
    pub environment: EnvironmentChecker<'a, MC, LR>,
}

impl<MC: MetadataChecks, LR: LinkReader> DotzoChecker<'_, MC, LR> {
    pub fn check(&self, dotzo: &Dotzo) -> Result<()> {
        self.environment.check(&dotzo.environment)?;
        Ok(())
    }
}
