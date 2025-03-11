use derive_more::derive::Constructor;
use thiserror::Error;

use crate::util::{
    actions::{Actions, Error as ActionError},
    dir::LabeledDir,
    prompting::{Prompter, PrompterError},
};

#[derive(Debug, Error)]
pub enum DirectoryCreatorError {
    #[error("Action error")]
    Action(#[from] ActionError),

    #[error("Prompt error")]
    Prompt(#[from] PrompterError),

    #[error("Declined to create")]
    DeclinedToCreate,
}

pub type Result<T> = core::result::Result<T, DirectoryCreatorError>;

#[derive(Debug, Constructor)]
pub struct DirectoryCreator<'a, A: Actions, PR: Prompter> {
    actions: &'a A,
    prompter: &'a PR,
}

impl<A: Actions, PR: Prompter> DirectoryCreator<'_, A, PR> {
    pub fn create<D: LabeledDir>(&self, dir: &D, yes: bool) -> Result<()> {
        if yes
            || self
                .prompter
                .confirm(format!("Create {} directory at {}", D::LABEL, dir), false)?
        {
            self.actions.make_dir(dir)?;
            Ok(())
        } else {
            Err(DirectoryCreatorError::DeclinedToCreate)
        }
    }
}
