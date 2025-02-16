use super::types::Environment;
use crate::{
    components::validation::{
        containment::{ContainmentCheck, ContainmentError},
        directory::{DirectoryCheck, DirectoryCheckError},
    },
    util::{
        actions::Actions,
        fs::{LinkReader, MetadataChecks},
        prompting::Prompter,
    },
};
use derive_more::derive::Constructor;
use log::error;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum EnvironmentCheckerError<E: std::error::Error> {
    #[error("Directory check failure")]
    DirectoryCheckFailure(#[from] DirectoryCheckError<E>),

    #[error("Containment check failure")]
    ContainmentCheckFailure(#[from] ContainmentError),
}

pub type Result<T, E> = std::result::Result<T, EnvironmentCheckerError<E>>;

#[derive(Debug, Constructor)]
pub struct EnvironmentChecker<'a, MC: MetadataChecks, LR: LinkReader, A: Actions, PR: Prompter> {
    checker: &'a DirectoryCheck<'a, MC, A, PR>,
    containment: &'a ContainmentCheck<'a, MC, LR>,
}

impl<MC: MetadataChecks, A: Actions, LR: LinkReader, PR: Prompter> EnvironmentChecker<'_, MC, LR, A, PR> {
    pub fn check(&self, environment: &Environment) -> Result<(), PR::Error> {
        self.checker.check(&environment.home, false)?;
        self.checker.check(&environment.config, true)?;
        self.containment
            .check(&environment.config, &environment.home)
            .inspect_err(|_| error!("The config directory is not inside of the home tree."))?;
        Ok(())
    }
}
