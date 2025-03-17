use derive_more::derive::Constructor;
use tryiter::TryIteratorExt;

use log::{debug, warn};
use std::{
    collections::HashSet,
    path::{Path, PathBuf},
};
use thiserror::Error;

use crate::{
    components::repo::directory::{DirVisitation, IgnoreType, RepoDirItem, RepoDirItemWithPath},
    config::{file::ConfigFileReadError, spec::translate::SpecContext},
    mapping::{DotMap, DotMaps},
    util::fs::{DirectoryListing, MetadataChecks},
};

use super::directory::RepoDirVisitorError;

fn hash_set_to_string(hs: &HashSet<String>) -> String {
    hs.iter()
        .map(|s| format!("[{}]", s))
        .collect::<Vec<_>>()
        .join(", ")
}

pub struct DirData {
    pub path: PathBuf,
    pub expected_targets: HashSet<String>,
    pub implicit_ignores: HashSet<String>,
}

impl DirData {
    pub fn new(path: PathBuf, context: &SpecContext) -> Self {
        Self {
            path,
            expected_targets: context.targets.keys().map(ToOwned::to_owned).collect(),
            implicit_ignores: Default::default(),
        }
    }

    pub fn report(&self) {
        if !self.implicit_ignores.is_empty() {
            warn!(
                "The following in {:?} were not mentioned in .dot, and are being ignored implicitly: {}",
                self.path.display(),
                hash_set_to_string(&self.implicit_ignores)
            );
        }

        if !self.expected_targets.is_empty() {
            warn!(
                "The .dot in {:?} referenced the following, but they were not found: {}",
                self.path.display(),
                hash_set_to_string(&self.expected_targets)
            );
        }
    }
}

#[derive(Debug, Error)]
pub enum TreeTraverserError {
    #[error("Io error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Error reading config file: {0}")]
    ConfigFile(#[from] ConfigFileReadError),

    #[error("Error visiting directory: {0}")]
    DirVisitor(#[from] RepoDirVisitorError),
}

pub type Result<T> = core::result::Result<T, TreeTraverserError>;

#[derive(Debug)]
pub struct TreeTraverser<'a, DL: DirectoryListing, MC: MetadataChecks> {
    visitor: DirVisitation<'a, MC, DL>,
}

#[derive(Constructor)]
struct DirectoryItemConsumer<'a, 'b> {
    mapping: &'a mut DotMaps,
    dir_data: &'b mut DirData,
}

impl DirectoryItemConsumer<'_, '_> {
    fn consume(
        &mut self,
        RepoDirItemWithPath { path, item }: RepoDirItemWithPath,
    ) -> Option<PathBuf> {
        match item {
            RepoDirItem::SubDir => Some(path),
            RepoDirItem::Mapping(name, located_target) => {
                let dot_map = DotMap::new(path.clone(), located_target);
                debug!("adding: {} => {:?}", name, dot_map);
                self.mapping.insert(path, dot_map);
                self.dir_data.expected_targets.remove(&name);
                None
            }
            RepoDirItem::Ignore(ig_type, name) => {
                debug!("ignoring({:?}): {}", ig_type, name);
                if ig_type == IgnoreType::Implicit {
                    self.dir_data.implicit_ignores.insert(name);
                }
                None
            }
        }
    }
}

impl<'a, DL: DirectoryListing, MC: MetadataChecks> TreeTraverser<'a, DL, MC> {
    pub fn new(metadata_checks: &'a MC, directory_listing: &'a DL) -> Self {
        Self {
            visitor: DirVisitation::new(metadata_checks, directory_listing),
        }
    }

    pub fn traverse(&self, root: impl AsRef<Path>) -> Result<DotMaps> {
        let mut mapping: DotMaps = Default::default();
        let mut stack = vec![root.as_ref().to_path_buf()];
        while let Some(current) = stack.pop() {
            debug!("Visiting directory: {:?}", current);
            let context = SpecContext::from_path(&current)?;
            let mut dir_data = DirData::new(current.clone(), &context);
            let mut consumer = DirectoryItemConsumer::new(&mut mapping, &mut dir_data);
            self.visitor
                .visit(&current, &context)?
                .try_filter_map(|item| Ok(consumer.consume(item)))
                .try_for_each(|entry| entry.map(|path| stack.push(path)))?;
            dir_data.report();
        }

        Ok(mapping)
    }
}
