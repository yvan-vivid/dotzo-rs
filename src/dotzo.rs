use derive_more::derive::Constructor;
use log::error;
use thiserror::Error;

use crate::components::{
    environment::types::{Environment, EnvironmentError},
    repo::types::Repo,
};

#[derive(Debug, Error)]
pub enum DotzoError {
    #[error("Environment Error")]
    Environment(#[from] EnvironmentError),
}

pub type Result<T> = std::result::Result<T, DotzoError>;

#[derive(Debug, Constructor, PartialEq, Eq)]
pub struct Dotzo {
    pub environment: Environment,
    pub repo: Repo,
}

impl Dotzo {
    pub fn check(&self) -> Result<()> {
        self.environment.check()?;
        Ok(())
    }
}
