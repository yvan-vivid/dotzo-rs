use derive_more::derive::{AsRef, Constructor, Display, From};
use relative_path::RelativePathBuf;
use std::path::{Path, PathBuf};

use crate::{dir, label, labeled_dir, mapping::Destination, util::dir::Labeled};

labeled_dir!(Home, "home");

pub trait CoreDir: Labeled + From<PathBuf> {
    fn relative_to_home() -> RelativePathBuf;

    fn from_home(home: &Home) -> PathBuf {
        Self::relative_to_home().to_path(home)
    }
}

macro_rules! core_dir {
    ($x:ident, $p:literal, $d:literal) => {
        labeled_dir!($x, $p);
        impl CoreDir for $x {
            fn relative_to_home() -> RelativePathBuf {
                RelativePathBuf::from($p)
            }
        }
    };
}

core_dir!(ConfigDir, "config", ".config");
core_dir!(DataDir, "data", ".local/share");
core_dir!(StateDir, "state", ".local/state");
core_dir!(CacheDir, "cache", ".cache");

#[derive(Debug, Constructor)]
pub struct DestinationData<'a> {
    pub dot_default: bool,
    pub path: &'a Path,
}

#[derive(Debug, Constructor, PartialEq, Eq)]
pub struct Environment {
    pub home: Home,
    pub config: ConfigDir,
    pub data: DataDir,
    pub state: StateDir,
    pub cache: CacheDir,
}

impl Environment {
    pub fn destination_data<'a>(&'a self, destination: &Destination) -> DestinationData<'a> {
        match destination {
            Destination::Home => DestinationData::new(true, self.home.as_ref()),
            Destination::Config => DestinationData::new(false, self.config.as_ref()),
        }
    }
}
