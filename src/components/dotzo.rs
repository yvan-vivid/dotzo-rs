use derive_more::derive::Constructor;
use log::error;
use thiserror::Error;

use crate::util::fs::{LinkReader, MetadataChecks};

use super::{
    environment::{
        checks::{EnvironmentChecker, EnvironmentCheckerError},
        types::Environment,
    },
    repo::types::{Repo, RepoError},
};

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
    pub fn check(&self, environment: &Environment, _repo: &Repo) -> Result<()> {
        self.environment.check(environment)?;
        Ok(())
    }
}
