use derive_more::derive::{Constructor, From};
use inquire::InquireError;
use log::error;
use std::path::{Path, PathBuf};
use thiserror::Error;

use crate::{
    config::file::ConfigFileReadError,
    mapping::Destination,
    util::{
        actions::Actions,
        checks::{DirectoryCheck, DirectoryCheckError},
        fs::MetadataChecks,
    },
};

#[derive(Debug, Error)]
pub enum EnvironmentError {
    #[error("Config file reading error reading config file")]
    ConfigFileReading(#[from] ConfigFileReadError),

    #[error("Prompting error")]
    Prompting(#[from] InquireError),

    #[error("IO error")]
    Io(#[from] std::io::Error),

    #[error("Directory check failure")]
    DirectoryCheckFailure(#[from] DirectoryCheckError),
}

pub type Result<T> = std::result::Result<T, EnvironmentError>;

#[derive(Debug, Constructor, PartialEq, From, Eq)]
pub struct Home(PathBuf);

#[derive(Debug, Constructor, PartialEq, From, Eq)]
pub struct Configs(PathBuf);

#[derive(Debug, Constructor)]
pub struct DestinationData<'a> {
    pub dot_default: bool,
    pub path: &'a Path,
}

#[derive(Debug, Constructor, PartialEq, Eq)]
pub struct Environment {
    pub home: Home,
    pub config: Configs,
}

impl AsRef<Path> for Home {
    fn as_ref(&self) -> &Path {
        self.0.as_path()
    }
}

impl AsRef<Path> for Configs {
    fn as_ref(&self) -> &Path {
        self.0.as_path()
    }
}

impl Environment {
    pub fn destination_data<'a>(&'a self, destination: &Destination) -> DestinationData<'a> {
        match destination {
            Destination::Home => DestinationData::new(true, self.home.as_ref()),
            Destination::Config => DestinationData::new(false, self.config.as_ref()),
        }
    }
}
