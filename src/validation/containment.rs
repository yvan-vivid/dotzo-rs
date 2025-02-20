use derive_more::Constructor;
use log::{debug, error};
use std::path::Path;
use thiserror::Error;

use crate::util::fs::{LinkReader, MetadataChecks};

#[derive(Debug, Error)]
pub enum ContainmentError {
    #[error("Io error")]
    Io(#[from] std::io::Error),

    #[error("The conainer is not a directory")]
    ContainerNotADirectory,

    #[error("The path is not in container")]
    NotContained,
}

pub type Result<T> = core::result::Result<T, ContainmentError>;

#[derive(Debug, Constructor)]
pub struct ContainmentCheck<'a, MC: MetadataChecks, LR: LinkReader> {
    metadata_checks: &'a MC,
    link_reader: &'a LR,
}

impl<MC: MetadataChecks, LR: LinkReader> ContainmentCheck<'_, MC, LR> {
    pub fn check<P: AsRef<Path>, Q: AsRef<Path>>(&self, path: P, container: Q) -> Result<()> {
        let raw_path = path.as_ref();
        let raw_container = container.as_ref();

        let path = self.link_reader.canonicalize(raw_path)?;
        debug!("Canonicalized path {} to {}", raw_path.display(), path.display());

        let container = self.link_reader.canonicalize(container.as_ref())?;
        debug!(
            "Canonicalized container {} to {}",
            raw_container.display(),
            container.display()
        );

        if !self.metadata_checks.is_dir(&container) {
            debug!("Container {} is not a directory", container.display());
            return Err(ContainmentError::ContainerNotADirectory);
        }

        if !path.starts_with(&container) {
            debug!(
                "Container {} is not a prefix of path {}",
                container.display(),
                path.display()
            );
            return Err(ContainmentError::NotContained);
        }

        Ok(())
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

    static TEST_CONTAINMENT_CHECKER: LazyLock<ContainmentCheck<'_, TestFs, TestFs>> =
        LazyLock::new(|| ContainmentCheck::new(&*TEST_FS, &*TEST_FS));

    #[test]
    fn test_check_contained() {
        let checked = TEST_CONTAINMENT_CHECKER.check(PathBuf::from("path/to/directory"), PathBuf::from("path"));
        assert!(checked.is_ok());
    }

    #[test]
    fn test_check_container_not_directory() {
        let checked = TEST_CONTAINMENT_CHECKER.check(PathBuf::from("path/to/directory"), PathBuf::from("path/to/file"));
        assert!(matches!(checked, Err(ContainmentError::ContainerNotADirectory)));
    }

    #[test]
    fn test_check_directory_not_contained() {
        let checked = TEST_CONTAINMENT_CHECKER.check(PathBuf::from("path/to/file"), PathBuf::from("path/to/directory"));
        assert!(matches!(checked, Err(ContainmentError::NotContained)));
    }

    // TODO: Cover canonicalize errors
}
