use std::io::ErrorKind;

use derive_more::derive::Constructor;
use log::{debug, info};
use thiserror::Error;

use crate::{
    components::linker::types::DotLink,
    util::{
        actions::{ActionError, Actions},
        fs::{LinkReader, MetadataChecks},
    },
};

#[derive(Debug, Error)]
pub enum LinkCreatorError {
    #[error("Action error")]
    Action(#[from] ActionError),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}

pub type Result<T> = core::result::Result<T, LinkCreatorError>;

#[derive(Debug, Constructor)]
pub struct LinkCreator<'a, MC: MetadataChecks, LR: LinkReader, A: Actions> {
    metadata_checks: &'a MC,
    link_reader: &'a LR,
    actions: &'a A,
}

impl<MC: MetadataChecks, LR: LinkReader, A: Actions> LinkCreator<'_, MC, LR, A> {
    pub fn create(&self, DotLink { target, link }: &DotLink) -> Result<bool> {
        let link_path = &link.to_path("");

        debug!(
            "Attempting to make link {} => {}",
            target.display(),
            link_path.display()
        );

        match self
            .actions
            .symlink(target, link_path)
            .inspect(|_| info!("Linked {} => {}", target.display(), link_path.display()))
        {
            Ok(()) => Ok(true),
            Err(ActionError::Io(ioe)) if ioe.kind() == ErrorKind::AlreadyExists => {
                if self.metadata_checks.is_symlink(target) {
                    let current_link = self.link_reader.read_link(target)?;
                    if &current_link == link_path {
                        debug!(
                            "Link {} alread exists pointing to {}",
                            target.display(),
                            current_link.display()
                        );
                        return Ok(false);
                    }
                    debug!(
                        "Link {} alread exists, but points to {}",
                        target.display(),
                        current_link.display()
                    );
                } else {
                    debug!("A file already exists at {}", target.display());
                }

                // TODO: Collision handling
                Err(ActionError::from_io_kind(ErrorKind::AlreadyExists).into())
            }
            Err(e) => Err(e.into()),
        }
    }
}
