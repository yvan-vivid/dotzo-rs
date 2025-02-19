use super::types::{Environment, Home};
use crate::{
    action::directory_creator::{DirectoryCreator, DirectoryCreatorError},
    util::{
        actions::Actions,
        dir::LabeledDir,
        fs::{LinkReader, MetadataChecks},
        prompting::Prompter,
    },
    validation::{
        containment::{ContainmentCheck, ContainmentError},
        directory::{DirectoryCheck, DirectoryCheckError},
    },
};
use derive_more::derive::Constructor;
use log::{error, info};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum EnvironmentCheckerError {
    #[error("Directory check failure")]
    DirectoryCheck(#[from] DirectoryCheckError),

    #[error("Containment check failure")]
    ContainmentCheck(#[from] ContainmentError),

    #[error("Directory creation failure")]
    DirectoryCreator(#[from] DirectoryCreatorError),
}

pub type Result<T> = std::result::Result<T, EnvironmentCheckerError>;

#[derive(Debug, Constructor)]
pub struct EnvironmentChecker<'a, MC: MetadataChecks, LR: LinkReader, A: Actions, PR: Prompter> {
    pub directory_checker: DirectoryCheck<'a, MC>,
    pub containment: ContainmentCheck<'a, MC, LR>,
    pub directory_creator: DirectoryCreator<'a, A, PR>,
    yes: bool,
    create_directories: bool,
}

impl<MC: MetadataChecks, LR: LinkReader, A: Actions, PR: Prompter> EnvironmentChecker<'_, MC, LR, A, PR> {
    fn check_or_create<D: LabeledDir>(&self, dir: &D) -> Result<()> {
        self.directory_checker.check(dir).or_else(|e| match e {
            DirectoryCheckError::DoesNotExist if self.create_directories => {
                info!("Will try and create {}", dir);
                Ok(self.directory_creator.create(dir, self.yes)?)
            }
            e => Err(EnvironmentCheckerError::DirectoryCheck(e)),
        })
    }

    fn check_containment<D: LabeledDir>(&self, home: &Home, dir: &D) -> Result<()> {
        Ok(self
            .containment
            .check(dir, home)
            .inspect_err(|_| error!("The {} directory is not inside of the home tree.", D::LABEL))?)
    }

    pub fn check_structure(&self, environment: &Environment) -> Result<()> {
        self.check_containment(&environment.home, &environment.config)?;
        self.check_containment(&environment.home, &environment.data)?;
        self.check_containment(&environment.home, &environment.state)?;
        self.check_containment(&environment.home, &environment.cache)?;
        Ok(())
    }

    pub fn check_tree(&self, environment: &Environment) -> Result<()> {
        self.check_or_create(&environment.config)?;
        self.check_or_create(&environment.data)?;
        self.check_or_create(&environment.state)?;
        self.check_or_create(&environment.cache)?;
        Ok(())
    }
}
