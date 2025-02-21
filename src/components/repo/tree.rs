use log::{debug, warn};
use std::{
    collections::HashSet,
    path::{Path, PathBuf},
};
use thiserror::Error;

use crate::{
    components::repo::directory::{DirVisitor, IgnoreType, RepoDirItem, RepoDirItemWithPath},
    config::{file::ConfigFileReadError, spec::translate::SpecContext},
    mapping::{DotMap, DotMaps},
    util::fs::{DirectoryListing, MetadataChecks},
};

use super::directory::RepoDirVisitorError;

fn hash_set_to_string(hs: &HashSet<String>) -> String {
    hs.iter().map(|s| format!("[{}]", s)).collect::<Vec<_>>().join(", ")
}

pub struct DirData {
    pub path: PathBuf,
    pub expected_targets: HashSet<String>,
    pub implicit_ignores: HashSet<String>,
}

impl DirData {
    pub fn new<P: AsRef<Path>>(path: P, context: &SpecContext) -> Self {
        Self {
            path: path.as_ref().to_path_buf(),
            expected_targets: context.targets.keys().map(String::from).collect(),
            implicit_ignores: Default::default(),
        }
    }

    pub fn report(&self) {
        if !self.implicit_ignores.is_empty() {
            warn!(
                "The following in {:?} were not mentioned in .dot, and are being ignored implicitly.: {}",
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
    directory_listing: &'a DL,
    metadata_checks: &'a MC,
}

fn consume_directory_item(
    RepoDirItemWithPath { path, item }: RepoDirItemWithPath,
    mapping: &mut DotMaps,
    dir_data: &mut DirData,
) -> Option<PathBuf> {
    match item {
        RepoDirItem::SubDir => Some(path),
        RepoDirItem::Mapping(name, located_target) => {
            let dot_map = DotMap::new(path.clone(), located_target);
            debug!("adding: {} => {:?}", name, dot_map);
            mapping.insert(path, dot_map);
            dir_data.expected_targets.remove(&name);
            None
        }
        RepoDirItem::Ignore(ig_type, name) => {
            debug!("ignoring({:?}): {}", ig_type, name);
            if ig_type == IgnoreType::Implicit {
                dir_data.implicit_ignores.insert(name);
            }
            None
        }
    }
}

impl<'b, 'a: 'b, DL: DirectoryListing, MC: MetadataChecks> TreeTraverser<'a, DL, MC> {
    pub fn new(directory_listing: &'a DL, metadata_checks: &'a MC) -> Self {
        Self {
            directory_listing,
            metadata_checks,
        }
    }

    fn visit_directory<P: AsRef<Path>>(
        &self,
        path: P,
        context: &'b SpecContext,
    ) -> Result<DirVisitor<'b, 'a, DL::Iter, MC>> {
        Ok(DirVisitor::from_path(
            path,
            context,
            self.metadata_checks,
            self.directory_listing,
        )?)
    }

    pub fn traverse<P: AsRef<Path>>(&self, root: P) -> Result<DotMaps> {
        let mut mapping: DotMaps = Default::default();
        let mut stack = vec![root.as_ref().to_path_buf()];
        while let Some(current) = stack.pop() {
            debug!("Visiting directory: {:?}", current);
            let context = SpecContext::from_path(&current)?;
            let mut dir_data = DirData::new(&current, &context);
            for entry in self.visit_directory(&current, &context)?.filter_map(|visited| {
                visited
                    .map(|item| consume_directory_item(item, &mut mapping, &mut dir_data))
                    .transpose()
            }) {
                match entry {
                    Ok(path) => stack.push(path),
                    Err(e) => return Err(e.into()),
                }
            }
            dir_data.report();
        }

        Ok(mapping)
    }
}
