use derive_more::derive::Constructor;
use relative_path::{FromPathError, PathExt, RelativePathBuf, RelativeToError};
use thiserror::Error;

use crate::{
    components::environment::types::Environment,
    mapping::DotMap,
    util::fs::{LinkReader, MetadataChecks},
};

use super::types::{DotLink, DotStatus};

#[derive(Debug, Error)]
pub enum DotLinkerError {
    #[error("Relative path error: {0}")]
    FromPath(#[from] FromPathError),

    #[error("Relative path error: {0}")]
    RelativePath(#[from] RelativeToError),

    #[error("Io error: {0}")]
    Io(#[from] std::io::Error),
}

pub type Result<T> = core::result::Result<T, DotLinkerError>;

#[derive(Debug, Constructor)]
pub struct DotLinker<'a, MC: MetadataChecks, LR: LinkReader> {
    metadata_checks: &'a MC,
    link_reader: &'a LR,
}

impl<MC: MetadataChecks, LR: LinkReader> DotLinker<'_, MC, LR> {
    pub fn create_link(&self, environment: &Environment, map: &DotMap) -> Result<DotLink> {
        let source_path = self.link_reader.canonicalize(&map.source)?;
        let data = environment.destination_data(&map.target.destination);
        let target_directory = data.path;
        let target_path = target_directory.join(map.target.target.resolve(data.dot_default));
        let link_path = source_path.relative_to(target_directory)?;
        Ok(DotLink::new(target_path, link_path))
    }

    pub fn check(&self, link: &DotLink) -> Result<DotStatus> {
        if !self.metadata_checks.exists(&link.target) {
            return Ok(DotStatus::Pending);
        }

        if !self.metadata_checks.is_symlink(&link.target) {
            return Ok(DotStatus::Clobber);
        }

        let linked = self.link_reader.read_link(&link.target)?;

        if linked.is_absolute() {
            // TODO: handle wrong absolute link case
            return Ok(DotStatus::AbsoluteLink(linked));
        }

        let rel_linked = RelativePathBuf::from_path(&linked)?.normalize();
        if rel_linked != link.link {
            return Ok(DotStatus::WrongLink(rel_linked));
        }

        Ok(DotStatus::Confirmed)
    }
}
