use derive_more::derive::{Constructor, From};
use std::path::{Path, PathBuf};

use crate::mapping::Destination;

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
