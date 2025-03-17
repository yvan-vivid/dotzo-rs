use derive_more::derive::Constructor;
use std::path::{Path, PathBuf};
use thiserror::Error;

use crate::{config::spec::translate::SpecContext, mapping::LocatedTarget};

use crate::util::fs::{DirEntryIterator, DirectoryListing, MetadataChecks};

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum IgnoreType {
    Explicit,
    Implicit,
    Dot,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum RepoDirItem {
    Mapping(String, LocatedTarget),
    Ignore(IgnoreType, String),
    SubDir,
}

#[derive(Debug, Constructor, PartialEq, Eq, Clone)]
pub struct RepoDirItemWithPath {
    pub path: PathBuf,
    pub item: RepoDirItem,
}

#[derive(Error, Debug)]
pub enum RepoDirVisitorError {
    #[error("General io error")]
    Io(#[from] std::io::Error),

    #[error("Can't get file name from path: {0:?}")]
    CannotGetFileName(PathBuf),
}

pub type Result<V> = std::result::Result<V, RepoDirVisitorError>;

#[derive(Constructor)]
pub struct DirVisitor<'a, 'b: 'a, ID: DirEntryIterator, MC: MetadataChecks> {
    contents: ID,
    context: &'a SpecContext,
    metadata_checks: &'b MC,
}

impl<'a, 'b: 'a, ID: DirEntryIterator, MC: MetadataChecks> DirVisitor<'a, 'b, ID, MC> {
    fn visit(&self, path: impl AsRef<Path>) -> Result<RepoDirItem> {
        let path = path.as_ref();
        path.file_name()
            .and_then(|n| n.to_str())
            .ok_or_else(|| RepoDirVisitorError::CannotGetFileName(path.to_path_buf()))
            .map(ToOwned::to_owned)
            .map(|file_name| {
                if file_name.starts_with(".") {
                    RepoDirItem::Ignore(IgnoreType::Dot, file_name)
                } else if self.context.ignores.contains(&file_name) {
                    RepoDirItem::Ignore(IgnoreType::Explicit, file_name)
                } else if let Some(target) = self.context.targets.get(&file_name) {
                    RepoDirItem::Mapping(file_name, target.clone())
                } else if !self.metadata_checks.is_real_dir(path) {
                    RepoDirItem::Ignore(IgnoreType::Implicit, file_name)
                } else {
                    RepoDirItem::SubDir
                }
            })
    }
}

impl<ID: DirEntryIterator, MC: MetadataChecks> Iterator for DirVisitor<'_, '_, ID, MC> {
    type Item = Result<RepoDirItemWithPath>;

    fn next(&mut self) -> Option<Self::Item> {
        self.contents.next().map(|e| {
            e.map_err(|i| i.into()).and_then(|path| {
                self.visit(&path)
                    .map(|item| RepoDirItemWithPath::new(path, item))
            })
        })
    }
}

#[derive(Debug, Constructor)]
pub struct DirVisitation<'a, MC: MetadataChecks, DL: DirectoryListing> {
    metadata_checks: &'a MC,
    directory_listing: &'a DL,
}

impl<'b, MC: MetadataChecks, DL: DirectoryListing> DirVisitation<'b, MC, DL> {
    pub fn visit<'a, P: AsRef<Path>>(
        &self,
        path: P,
        context: &'a SpecContext,
    ) -> Result<DirVisitor<'a, 'b, DL::Iter, MC>> {
        Ok(DirVisitor::new(
            self.directory_listing.read_dir(path)?,
            context,
            self.metadata_checks,
        ))
    }
}

#[cfg(test)]
mod test {
    use std::sync::LazyLock;

    use super::*;
    use crate::{
        config::spec::types::{Mapping, Shorthand, Spec},
        mapping::Target,
        util::fs::{
            testing::{TestFile, TestFs},
            DirEntryResult,
        },
    };

    static TEST_SPEC: LazyLock<Spec> = LazyLock::new(|| {
        Spec::new(
            Some(vec![
                Shorthand::Name("in_home".into()),
                Shorthand::Mapped(Mapping {
                    source: "source_name".into(),
                    target: Some("target_name".into()),
                    dot: None,
                }),
            ]),
            Some(vec![Shorthand::Mapped(Mapping {
                source: "original_name".into(),
                target: None,
                dot: Some(false),
            })]),
            Some(["ignore_a".into(), "ignore_b".into()].into_iter().collect()),
        )
    });

    static TEST_TREE: LazyLock<TestFs> = LazyLock::new(|| {
        TestFs::new([
            (PathBuf::from("path/to/in_home"), TestFile::Regular),
            (PathBuf::from("path/to/source_name"), TestFile::Regular),
            (PathBuf::from("path/to/target_name"), TestFile::Regular),
            (PathBuf::from("path/to/original_name"), TestFile::Regular),
            (PathBuf::from("path/to/ignore_a"), TestFile::Regular),
            (PathBuf::from("path/to/ignore_b"), TestFile::Regular),
            (PathBuf::from("path/to/dir-x"), TestFile::Directory),
            (PathBuf::from("path/to/dir-y"), TestFile::Directory),
        ])
    });

    #[test]
    fn test_dir_visitor_empty() {
        let test_spec = Spec {
            home: None,
            config: None,
            ignore: None,
        };
        let test_entries = vec![];
        let expected: Vec<RepoDirItemWithPath> = vec![];

        let test_context = SpecContext::new(test_spec);
        let visitor = DirVisitor::new(test_entries.into_iter(), &test_context, &*TEST_TREE);
        let result = visitor.collect::<Result<Vec<_>>>();
        assert!(matches!(result, Ok(actual) if actual == expected));
    }

    #[test]
    fn test_dir_visitor() {
        let test_entries: Vec<DirEntryResult> = vec![
            Ok("path/to/in_home".into()),
            Ok("path/to/ignore_a".into()),
            Ok("path/to/dir-x".into()),
            Ok("path/to/source_name".into()),
            Ok("path/to/original_name".into()),
            Ok("path/to/dir-y".into()),
            Ok("path/to/unnamed".into()),
        ];
        let expected: Vec<RepoDirItemWithPath> = vec![
            RepoDirItemWithPath::new(
                "path/to/in_home".into(),
                RepoDirItem::Mapping(
                    "in_home".into(),
                    LocatedTarget::new(
                        Target::new("in_home".into(), None),
                        crate::mapping::Destination::Home,
                    ),
                ),
            ),
            RepoDirItemWithPath::new(
                "path/to/ignore_a".into(),
                RepoDirItem::Ignore(IgnoreType::Explicit, "ignore_a".into()),
            ),
            RepoDirItemWithPath::new("path/to/dir-x".into(), RepoDirItem::SubDir),
            RepoDirItemWithPath::new(
                "path/to/source_name".into(),
                RepoDirItem::Mapping(
                    "source_name".into(),
                    LocatedTarget::new(
                        Target::new("target_name".into(), None),
                        crate::mapping::Destination::Home,
                    ),
                ),
            ),
            RepoDirItemWithPath::new(
                "path/to/original_name".into(),
                RepoDirItem::Mapping(
                    "original_name".into(),
                    LocatedTarget::new(
                        Target::new("original_name".into(), Some(false)),
                        crate::mapping::Destination::Config,
                    ),
                ),
            ),
            RepoDirItemWithPath::new("path/to/dir-y".into(), RepoDirItem::SubDir),
            RepoDirItemWithPath::new(
                "path/to/unnamed".into(),
                RepoDirItem::Ignore(IgnoreType::Implicit, "unnamed".into()),
            ),
        ];

        let test_context = SpecContext::new(TEST_SPEC.clone());
        let visitor = DirVisitor::new(test_entries.into_iter(), &test_context, &*TEST_TREE);
        let result = visitor.collect::<Result<Vec<_>>>();
        assert!(matches!(result, Ok(actual) if actual == expected));
    }

    #[test]
    fn test_dir_visitor_error() {
        let test_entries: Vec<DirEntryResult> = vec![
            Ok("path/to/in_home".into()),
            Ok("path/to/ignore_a".into()),
            Err(std::io::Error::other("an io error")),
        ];

        let test_context = SpecContext::new(TEST_SPEC.clone());
        let visitor = DirVisitor::new(test_entries.into_iter(), &test_context, &*TEST_TREE);
        let result = visitor.collect::<Result<Vec<_>>>();
        assert!(matches!(
            result,
            Err(RepoDirVisitorError::Io(e)) 
            if e.kind() == std::io::ErrorKind::Other && e.to_string() == "an io error"));
    }
}
