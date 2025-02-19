use derive_more::derive::Constructor;
use log::error;
use thiserror::Error;

use crate::util::actions::Actions;
use crate::util::fs::{LinkReader, MetadataChecks};

use crate::components::{
    environment::checks::{EnvironmentChecker, EnvironmentCheckerError},
    repo::types::RepoError,
};
use crate::util::prompting::Prompter;

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
pub struct DotzoChecker<'a, MC: MetadataChecks, LR: LinkReader, A: Actions, PR: Prompter> {
    pub environment: EnvironmentChecker<'a, MC, LR, A, PR>,
}

impl<MC: MetadataChecks, LR: LinkReader, A: Actions, PR: Prompter> DotzoChecker<'_, MC, LR, A, PR> {
    pub fn check(&self, dotzo: &Dotzo) -> Result<()> {
        self.environment.check_tree(&dotzo.environment)?;
        Ok(())
    }
}
