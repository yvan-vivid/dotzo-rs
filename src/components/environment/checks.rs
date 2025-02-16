use super::types::Environment;
use crate::{
    components::validation::{
        containment::{ContainmentCheck, ContainmentError},
        directory::{DirectoryCheck, DirectoryCheckError},
    },
    util::fs::{LinkReader, MetadataChecks},
};
use derive_more::derive::Constructor;
use log::error;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum EnvironmentCheckerError {
    #[error("Directory check failure")]
    DirectoryCheckFailure(#[from] DirectoryCheckError),

    #[error("Containment check failure")]
    ContainmentCheckFailure(#[from] ContainmentError),
}

pub type Result<T> = std::result::Result<T, EnvironmentCheckerError>;

#[derive(Debug, Constructor)]
pub struct EnvironmentChecker<'a, MC: MetadataChecks, LR: LinkReader> {
    checker: &'a DirectoryCheck<'a, MC>,
    containment: &'a ContainmentCheck<'a, MC, LR>,
}

impl<MC: MetadataChecks, LR: LinkReader> EnvironmentChecker<'_, MC, LR> {
    pub fn check(&self, environment: &Environment) -> Result<()> {
        self.checker.check(&environment.home)?;
        self.checker.check(&environment.config)?;
        self.containment
            .check(&environment.config, &environment.home)
            .inspect_err(|_| error!("The config directory is not inside of the home tree."))?;
        Ok(())
    }
}
