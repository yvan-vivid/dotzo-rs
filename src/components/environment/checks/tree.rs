use std::path::PathBuf;

use derive_more::derive::Constructor;
use log::{debug, error};
use thiserror::Error;

use crate::{
    action::directory_creator::{DirectoryCreator, DirectoryCreatorError},
    components::environment::types::Environment,
    util::{actions::Actions, dir::LabeledDir, fs::MetadataChecks, prompting::Prompter},
    validation::directory::{DirectoryCheck, DirectoryCheckError},
};

#[derive(Debug, Error)]
pub enum LayoutCheckError {
    #[error("Directory check for {label} failed at {path} with {error:?}")]
    Check {
        label: &'static str,
        path: PathBuf,
        error: DirectoryCheckError,
    },

    #[error("Directory creation for {label} failed at {path} with {error:?}")]
    Creation {
        label: &'static str,
        path: PathBuf,
        error: DirectoryCreatorError,
    },
}

pub type Result<T> = std::result::Result<T, LayoutCheckError>;

#[derive(Debug, Constructor)]
pub struct LayoutCheck<'a, MC: MetadataChecks, A: Actions, PR: Prompter> {
    pub directory_checker: DirectoryCheck<'a, MC>,
    pub directory_creator: DirectoryCreator<'a, A, PR>,
    yes: bool,
    create_directories: bool,
}

impl<MC: MetadataChecks, A: Actions, PR: Prompter> LayoutCheck<'_, MC, A, PR> {
    fn check_or_create<D: LabeledDir>(&self, dir: &D) -> Result<()> {
        self.directory_checker.check(dir).or_else(|e| match e {
            DirectoryCheckError::DoesNotExist if self.create_directories => {
                debug!("Will try and create {}", dir);
                self.directory_creator
                    .create(dir, self.yes)
                    .map_err(|error| LayoutCheckError::Creation {
                        label: D::LABEL,
                        path: dir.as_ref().to_owned(),
                        error,
                    })
            }
            error => Err(LayoutCheckError::Check {
                label: D::LABEL,
                path: dir.as_ref().to_owned(),
                error,
            }),
        })
    }

    pub fn check(&self, environment: &Environment) -> Result<()> {
        self.check_or_create(&environment.config)?;
        self.check_or_create(&environment.data)?;
        self.check_or_create(&environment.state)?;
        self.check_or_create(&environment.cache)?;
        Ok(())
    }
}
