use derive_more::derive::Constructor;
use relative_path::RelativePathBuf;
use std::{collections::HashSet, path::PathBuf};

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
