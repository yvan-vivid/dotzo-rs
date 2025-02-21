use derive_more::derive::Constructor;
use log::error;
use std::path::PathBuf;
use thiserror::Error;

use crate::{
    components::environment::types::{Environment, Home},
    util::{
        dir::LabeledDir,
        fs::{LinkReader, MetadataChecks},
    },
    validation::containment::{ContainmentCheck, ContainmentError},
};

#[derive(Debug, Error)]
pub enum StructureCheckError {
    #[error("Containment check for {label} failed at {path} with {error:?}")]
    Containment {
        label: &'static str,
        path: PathBuf,
        error: ContainmentError,
    },
}

pub type Result<T> = std::result::Result<T, StructureCheckError>;

#[derive(Debug, Constructor)]
pub struct StructureCheck<'a, MC: MetadataChecks, LR: LinkReader> {
    pub containment: ContainmentCheck<'a, MC, LR>,
}

impl<MC: MetadataChecks, LR: LinkReader> StructureCheck<'_, MC, LR> {
    fn check_containment<D: LabeledDir>(&self, home: &Home, dir: &D) -> Result<()> {
        self.containment
            .check(dir, home)
            .map_err(|error| StructureCheckError::Containment {
                label: D::LABEL,
                path: dir.as_ref().to_owned(),
                error,
            })
    }

    pub fn check(&self, environment: &Environment) -> Result<()> {
        self.check_containment(&environment.home, &environment.config)?;
        self.check_containment(&environment.home, &environment.data)?;
        self.check_containment(&environment.home, &environment.state)?;
        self.check_containment(&environment.home, &environment.cache)?;
        Ok(())
    }
}
