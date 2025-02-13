use derive_more::derive::Constructor;
use inquire::InquireError;
use log::{debug, error};
use thiserror::Error;

use crate::{
    components::{
        environment::{
            home::Home,
            types::{Environment, EnvironmentError},
        },
        repo::types::Repo,
    },
    config::{
        file::{ConfigFileReadError, ReadFromConfig},
        rc::types::Rc,
    },
};

#[derive(Debug, Error)]
pub enum DotzoError {
    #[error("Config file reading error reading config file")]
    ConfigFileReading(#[from] ConfigFileReadError),

    #[error("Prompting error")]
    Prompting(#[from] InquireError),

    #[error("IO error")]
    Io(#[from] std::io::Error),

    #[error("Config file not found")]
    ConfigFileNotFound,

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
    pub fn from_config(home: Home, rc: Rc) -> Result<Self> {
        let repo_path = rc.repo.location.to_path(&home);
        let environment = Environment::new(home);
        let repo = Repo::new(repo_path);
        Ok(Dotzo { environment, repo })
    }

    pub fn from_config_path(home: Home) -> Result<Self> {
        debug!("Looking for a config in home: {}", home.as_ref().display());
        let rc = Rc::find_in_path(&home)?.ok_or_else(|| DotzoError::ConfigFileNotFound)?;
        Self::from_config(home, rc)
    }

    pub fn check(&self) -> Result<()> {
        self.environment.check()?;
        Ok(())
    }
}
