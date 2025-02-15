use anyhow::Result;
use derive_more::derive::Constructor;
use relative_path::{PathExt, RelativePathBuf};
use std::{collections::HashSet, path::PathBuf};

use crate::{
    mapping::DotMap,
    util::fs::{LinkReader, MetadataChecks},
};

use super::environment::types::Environment;

#[derive(Debug, PartialEq, Eq)]
pub enum DotStatus {
    // DotMap already correct
    Confirmed,

    // DotMap can be created without issue
    Pending,

    // DotMap target is already there and not a link
    Clobber,

    // DotMap target is already there but points to a different source
    WrongLink(RelativePathBuf),

    // DotMap target points to the right file but with an absolute link
    AbsoluteLink(PathBuf),

    // DotMap target is already there but points to a different source
    WrongAbsoluteLink(PathBuf),
}

#[derive(Debug, Constructor, PartialEq, Eq, Hash)]
pub struct DotLink {
    // Absolute link to the target
    pub target: PathBuf,

    // Link to create
    pub link: RelativePathBuf,
}

pub type DotLinkSet = HashSet<DotLink>;

#[derive(Debug, Default)]
pub struct DotReconciliation {
    pub confirmed: DotLinkSet,
    pub pending: DotLinkSet,
    pub clobber: DotLinkSet,
    pub fix: DotLinkSet,
}

#[derive(Debug, Constructor)]
pub struct DotLinker<'a, MC: MetadataChecks, LR: LinkReader> {
    metadata_checks: &'a MC,
    link_reader: &'a LR,
}

impl DotLink {
    pub fn create(environment: &Environment, map: &DotMap) -> Result<Self> {
        // TODO: does canonicalize need to be injected?
        let source_path = map.source.canonicalize()?;
        let data = environment.destination_data(&map.target.destination);
        let target_directory = data.path;
        let target_path = target_directory.join(map.target.target.resolve(data.dot_default));
        let link_path = source_path.relative_to(target_directory)?;
        Ok(Self::new(target_path, link_path))
    }
}

impl DotReconciliation {
    pub fn new() -> Self {
        Default::default()
    }
}

impl<MC: MetadataChecks, LR: LinkReader> DotLinker<'_, MC, LR> {
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

    pub fn reconciliation<I: IntoIterator<Item = DotMap>>(
        &self,
        environment: &Environment,
        dot_maps: I,
    ) -> Result<DotReconciliation> {
        let mut recon = DotReconciliation::new();
        for dot_map in dot_maps {
            let link = DotLink::create(environment, &dot_map)?;
            match self.check(&link)? {
                DotStatus::Confirmed => recon.confirmed.insert(link),
                DotStatus::Pending => recon.pending.insert(link),
                DotStatus::Clobber => recon.clobber.insert(link),
                DotStatus::WrongLink(_relative_path_buf) => recon.fix.insert(link),
                DotStatus::AbsoluteLink(_path_buf) => recon.fix.insert(link),
                DotStatus::WrongAbsoluteLink(_path_buf) => recon.fix.insert(link),
            };
        }
        Ok(recon)
    }
}
