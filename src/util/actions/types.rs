use std::{io::ErrorKind, path::Path};

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("IO error")]
    Io(#[from] std::io::Error),
}

impl Error {
    pub fn from_io_kind(kind: ErrorKind) -> Self {
        std::io::Error::from(kind).into()
    }
}

pub type Result<T> = core::result::Result<T, Error>;

pub trait Actions {
    fn make_dir(&self, path: impl AsRef<Path>) -> Result<()>;
    fn symlink(&self, target: impl AsRef<Path>, path: impl AsRef<Path>) -> Result<()>;
}
