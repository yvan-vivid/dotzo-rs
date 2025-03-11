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
    fn canonicalize<P: AsRef<Path>>(&self, path: P) -> std::io::Result<PathBuf>;
}

pub trait FsRead: MetadataChecks + DirectoryListing + LinkReader {}
impl<T: MetadataChecks + DirectoryListing + LinkReader> FsRead for T {}

#[derive(Debug, Constructor)]
pub struct StandardFsRead {}

impl MetadataChecks for StandardFsRead {
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

impl DirectoryListing for StandardFsRead {
    type Iter = Box<dyn DirEntryIterator>;
    fn read_dir<P: AsRef<Path>>(&self, path: P) -> std::io::Result<Self::Iter> {
        Ok(Box::new(
            path.as_ref()
                .read_dir()?
                .map(|res| res.map(|dir_entry| dir_entry.path())),
        ))
    }
}

impl LinkReader for StandardFsRead {
    fn read_link<P: AsRef<Path>>(&self, path: P) -> std::io::Result<PathBuf> {
        path.as_ref().read_link()
    }

    fn canonicalize<P: AsRef<Path>>(&self, path: P) -> std::io::Result<PathBuf> {
        path.as_ref().canonicalize()
    }
}

#[cfg(test)]
pub mod testing {
    use std::collections::{HashMap, HashSet};
    use std::io::{Error, ErrorKind, Result};

    use super::*;

    #[derive(Debug, PartialEq, Eq, Hash, Clone)]
    pub enum TestFile {
        Regular,
        Directory,
        Symlink(PathBuf),
    }

    #[derive(Debug, PartialEq, Eq, Default)]
    pub struct TestFs {
        pub tree: HashMap<PathBuf, HashSet<PathBuf>>,
        pub files: HashMap<PathBuf, TestFile>,
    }

    impl TestFs {
        fn add_parents(&mut self, path: &Path) {
            let mut member = path.to_owned();
            while let Some(directory) = member.parent() {
                let d = directory.to_path_buf();
                self.tree
                    .entry(d.clone())
                    .or_default()
                    .insert(member.clone());
                member = d;
            }
        }

        pub fn add_file(&mut self, path: PathBuf, file: TestFile) {
            // TODO: Normalize path
            if let TestFile::Directory = file {
                self.tree.insert(path.clone(), Default::default());
            }
            self.add_parents(&path);
            self.files.insert(path, file);
        }

        pub fn add_directory<P: AsRef<Path>>(&mut self, path: P) {
            self.add_file(path.as_ref().to_owned(), TestFile::Directory);
        }

        pub fn new<I: IntoIterator<Item = (PathBuf, TestFile)>>(it: I) -> Self {
            let mut fs = TestFs::default();
            it.into_iter()
                .for_each(|(path, file)| fs.add_file(path, file));
            fs
        }

        pub fn get_file(&self, path: &PathBuf) -> std::io::Result<TestFile> {
            Ok(self
                .files
                .get(path)
                .cloned()
                .ok_or_else(|| Error::from(ErrorKind::NotFound))?)
        }

        pub fn follow_links<P: AsRef<Path>>(&self, path: P) -> std::io::Result<PathBuf> {
            let path = path.as_ref().to_path_buf();
            let mut visited: HashSet<PathBuf> = Default::default();
            visited.insert(path.to_path_buf());

            let mut current_path = path;
            let mut current = self.get_file(&current_path)?;
            while let TestFile::Symlink(linked) = current {
                if visited.contains(&linked) {
                    return Err(Error::from(ErrorKind::TooManyLinks));
                }
                visited.insert(linked.clone());
                current_path = linked;
                current = self.get_file(&current_path)?;
            }
            Ok(current_path)
        }
    }

    // TODO: Fix to cover symlink cases
    impl MetadataChecks for TestFs {
        fn is_dir<P: AsRef<Path>>(&self, path: P) -> bool {
            self.tree.contains_key(path.as_ref())
        }

        fn is_file<P: AsRef<Path>>(&self, path: P) -> bool {
            self.files.get(path.as_ref()).is_some_and(|tf| {
                if let TestFile::Regular = tf {
                    true
                } else {
                    false
                }
            })
        }

        fn is_symlink<P: AsRef<Path>>(&self, path: P) -> bool {
            if let Some(TestFile::Symlink(_)) = self.files.get(path.as_ref()) {
                true
            } else {
                false
            }
        }

        fn exists<P: AsRef<Path>>(&self, path: P) -> bool {
            self.files.contains_key(path.as_ref())
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
            self.follow_links(path)
        }

        fn canonicalize<P: AsRef<Path>>(&self, path: P) -> std::io::Result<PathBuf> {
            // TODO: Complete
            Ok(path.as_ref().to_path_buf())
        }
    }
}
