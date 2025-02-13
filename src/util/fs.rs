use std::path::{Path, PathBuf};

use derive_more::derive::Constructor;

pub type DirEntryResult = std::io::Result<PathBuf>;

pub trait DirEntryIterator: Iterator<Item = DirEntryResult> {}
impl<I: Iterator<Item = DirEntryResult>> DirEntryIterator for I {}

pub trait MetadataChecks {
    fn is_dir<P: AsRef<Path>>(&self, path: P) -> bool;
    fn is_file<P: AsRef<Path>>(&self, path: P) -> bool;
    fn is_symlink<P: AsRef<Path>>(&self, path: P) -> bool;
    fn exists<P: AsRef<Path>>(&self, path: P) -> bool;

    fn is_real_dir<P: AsRef<Path>>(&self, path: P) -> bool {
        self.is_dir(path.as_ref()) && !self.is_symlink(path.as_ref())
    }
}

pub trait DirectoryListing {
    type Iter: DirEntryIterator;
    fn read_dir<P: AsRef<Path>>(&self, path: P) -> std::io::Result<Self::Iter>;
}

pub trait LinkReader {
    fn read_link<P: AsRef<Path>>(&self, path: P) -> std::io::Result<PathBuf>;
}

pub trait Fs: MetadataChecks + DirectoryListing + LinkReader {}
impl<T: MetadataChecks + DirectoryListing + LinkReader> Fs for T {}

#[derive(Debug, Constructor)]
pub struct StandardFs {}

impl MetadataChecks for StandardFs {
    fn is_dir<P: AsRef<Path>>(&self, path: P) -> bool {
        path.as_ref().is_dir()
    }

    fn is_file<P: AsRef<Path>>(&self, path: P) -> bool {
        path.as_ref().is_file()
    }

    fn is_symlink<P: AsRef<Path>>(&self, path: P) -> bool {
        path.as_ref().is_symlink()
    }

    fn exists<P: AsRef<Path>>(&self, path: P) -> bool {
        path.as_ref().exists()
    }
}

impl DirectoryListing for StandardFs {
    type Iter = Box<dyn DirEntryIterator>;
    fn read_dir<P: AsRef<Path>>(&self, path: P) -> std::io::Result<Self::Iter> {
        Ok(Box::new(
            path.as_ref()
                .read_dir()?
                .map(|res| res.map(|dir_entry| dir_entry.path())),
        ))
    }
}

impl LinkReader for StandardFs {
    fn read_link<P: AsRef<Path>>(&self, path: P) -> std::io::Result<PathBuf> {
        path.as_ref().read_link()
    }
}

#[cfg(test)]
pub mod testing {
    use std::collections::{HashMap, HashSet};
    use std::io::{Error, ErrorKind, Result};

    use super::*;

    #[derive(Debug, PartialEq, Eq, Hash, Clone)]
    pub struct TestFile {
        pub name: String,
        pub is_file: bool,
        pub is_dir: bool,
    }

    #[derive(Debug, PartialEq, Eq)]
    pub struct TestFs {
        pub tree: HashMap<PathBuf, HashSet<PathBuf>>,
        pub files: HashMap<PathBuf, TestFile>,
        pub links: HashMap<PathBuf, PathBuf>,
    }

    impl TestFile {
        pub fn file(name: String) -> Self {
            Self {
                name,
                is_file: true,
                is_dir: false,
            }
        }

        pub fn directory(name: String) -> Self {
            Self {
                name,
                is_file: false,
                is_dir: true,
            }
        }
    }

    // TODO: Fix to cover symlink cases
    impl MetadataChecks for TestFs {
        fn is_dir<P: AsRef<Path>>(&self, path: P) -> bool {
            self.tree.contains_key(path.as_ref())
        }

        fn is_file<P: AsRef<Path>>(&self, path: P) -> bool {
            self.files.get(path.as_ref()).is_some_and(|tf| tf.is_file)
        }

        fn is_symlink<P: AsRef<Path>>(&self, path: P) -> bool {
            self.links.contains_key(path.as_ref())
        }

        fn exists<P: AsRef<Path>>(&self, path: P) -> bool {
            self.files.contains_key(path.as_ref()) || self.links.contains_key(path.as_ref())
        }
    }

    impl DirectoryListing for TestFs {
        type Iter = Box<dyn DirEntryIterator>;
        fn read_dir<P: AsRef<Path>>(&self, path: P) -> Result<Self::Iter> {
            let files = self
                .tree
                .get(path.as_ref())
                .ok_or_else(|| Error::new(ErrorKind::NotFound, "directory not found"))?;

            Ok(Box::new(files.clone().into_iter().map(Ok)))
        }
    }

    impl LinkReader for TestFs {
        fn read_link<P: AsRef<Path>>(&self, path: P) -> std::io::Result<PathBuf> {
            self.links
                .get(path.as_ref())
                .map(Clone::clone)
                .ok_or_else(|| Error::new(ErrorKind::NotFound, "directory not found"))
        }
    }
}
