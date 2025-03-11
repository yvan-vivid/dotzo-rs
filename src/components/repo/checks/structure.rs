use derive_more::derive::Constructor;
use log::error;
use thiserror::Error;

use crate::{
    components::repo::types::Repo,
    util::fs::MetadataChecks,
    validation::directory::{DirectoryCheck, DirectoryCheckError},
};

#[derive(Debug, Error)]
pub enum StructureCheckError {
    #[error("The given repo is missing")]
    Missing(DirectoryCheckError),

    #[error("<repo>/etc not found")]
    Etc(DirectoryCheckError),
}

pub type Result<T> = std::result::Result<T, StructureCheckError>;

#[derive(Debug, Constructor)]
pub struct StructureCheck<'a, MC: MetadataChecks> {
    pub exists: DirectoryCheck<'a, MC>,
    pub has_etc: DirectoryCheck<'a, MC>,
}

impl<MC: MetadataChecks> StructureCheck<'_, MC> {
    pub fn check(&self, repo: &Repo) -> Result<()> {
        self.exists
            .check(repo)
            .map_err(StructureCheckError::Missing)?;
        self.has_etc
            .check(repo.etc())
            .map_err(StructureCheckError::Etc)?;
        Ok(())
    }
}
