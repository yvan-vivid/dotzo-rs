use super::types::Environment;
use crate::util::{
    actions::Actions,
    checks::{DirectoryCheck, DirectoryCheckError},
    fs::MetadataChecks,
};
use derive_more::derive::Constructor;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum EnvironmentCheckerError {
    #[error("Directory check failure")]
    DirectoryCheckFailure(#[from] DirectoryCheckError),
}

pub type Result<T> = std::result::Result<T, EnvironmentCheckerError>;

#[derive(Debug, Constructor)]
pub struct EnvironmentChecker<'a, MC: MetadataChecks, A: Actions> {
    checker: &'a DirectoryCheck<'a, MC, A>,
}

impl<MC: MetadataChecks, A: Actions> EnvironmentChecker<'_, MC, A> {
    pub fn check(&self, environment: &Environment) -> Result<()> {
        self.checker.check(&environment.home, false)?;
        self.checker.check(&environment.config, true)?;
        Ok(())
    }
}
