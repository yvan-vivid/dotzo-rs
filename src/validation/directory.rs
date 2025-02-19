use derive_more::Constructor;
use log::error;
use std::path::Path;
use thiserror::Error;

use crate::util::fs::MetadataChecks;

#[derive(Debug, Error)]
pub enum DirectoryCheckError {
    #[error("Given directory does not exist")]
    DoesNotExist,

    #[error("Given directory is not a directory")]
    IsNotADirectory,
}

pub type Result<T> = core::result::Result<T, DirectoryCheckError>;

#[derive(Debug, Constructor)]
pub struct DirectoryCheck<'a, MC: MetadataChecks> {
    metadata_checks: &'a MC,
}

impl<MC: MetadataChecks> DirectoryCheck<'_, MC> {
    pub fn check<P: AsRef<Path>>(&self, path: P) -> Result<()> {
        let path = path.as_ref();
        if self.metadata_checks.is_dir(path) {
            return Ok(());
        }

        if self.metadata_checks.exists(path) {
            return Err(DirectoryCheckError::IsNotADirectory);
        }

        Err(DirectoryCheckError::DoesNotExist)
    }
}

#[cfg(test)]
mod test {
    use std::{path::PathBuf, sync::LazyLock};

    use super::*;
    use crate::util::fs::testing::{TestFile, TestFs};

    static TEST_FS: LazyLock<TestFs> = LazyLock::new(|| {
        TestFs::new([
            (PathBuf::from("path/to/directory"), TestFile::Directory),
            (PathBuf::from("path/to/file"), TestFile::Regular),
        ])
    });

    static TEST_DIRECTORY_CHECKER: LazyLock<DirectoryCheck<'_, TestFs>> =
        LazyLock::new(|| DirectoryCheck::new(&*TEST_FS));

    #[test]
    fn test_check_directory_exists() {
        let checked = TEST_DIRECTORY_CHECKER.check(PathBuf::from("path/to/directory"));
        assert!(checked.is_ok());
    }

    #[test]
    fn test_check_directory_not_a_directory() {
        let checked = TEST_DIRECTORY_CHECKER.check(PathBuf::from("path/to/file"));
        assert!(matches!(checked, Err(DirectoryCheckError::IsNotADirectory)));
    }

    #[test]
    fn test_check_directory_does_not_exist() {
        let checked = TEST_DIRECTORY_CHECKER.check(PathBuf::from("path/to/not-exist"));
        assert!(matches!(checked, Err(DirectoryCheckError::DoesNotExist)));
    }
}
