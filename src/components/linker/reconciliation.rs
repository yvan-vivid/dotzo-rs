use thiserror::Error;

use crate::{
    components::environment::types::Environment,
    mapping::DotMap,
    util::fs::{LinkReader, MetadataChecks},
};

use super::{
    link::{DotLinker, DotLinkerError},
    types::{DotLinkSet, DotStatus},
};

#[derive(Debug, Error)]
pub enum DotReconciliationError {
    #[error("Linker error: {0}")]
    Linker(#[from] DotLinkerError),
}

pub type Result<T> = core::result::Result<T, DotReconciliationError>;

#[derive(Debug, Default)]
pub struct DotReconciliation {
    pub confirmed: DotLinkSet,
    pub pending: DotLinkSet,
    pub clobber: DotLinkSet,
    pub fix: DotLinkSet,
}

impl DotReconciliation {
    pub fn with_linker<I: IntoIterator<Item = DotMap>, MC: MetadataChecks, LR: LinkReader>(
        linker: &DotLinker<'_, MC, LR>,
        environment: &Environment,
        dot_maps: I,
    ) -> Result<Self> {
        let mut recon = DotReconciliation::default();
        for dot_map in dot_maps {
            let link = linker.create_link(environment, &dot_map)?;
            match linker.check(&link)? {
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
