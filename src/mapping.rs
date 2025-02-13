use std::{collections::HashMap, path::PathBuf};

use derive_more::derive::Constructor;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Destination {
    Home,
    Config,
}

#[derive(Debug, Constructor, Clone, PartialEq, Eq)]
pub struct Target {
    pub name: String,
    pub dot: Option<bool>,
}

#[derive(Debug, Constructor, Clone, PartialEq, Eq)]
pub struct LocatedTarget {
    pub target: Target,
    pub destination: Destination,
}

#[derive(Debug, Constructor, Clone, PartialEq, Eq)]
pub struct DotMap {
    pub source: PathBuf,
    pub target: LocatedTarget,
}

pub type TargetMap = HashMap<String, LocatedTarget>;
pub type DotMaps = HashMap<PathBuf, DotMap>;

impl Destination {
    pub fn locate(&self, target: Target) -> LocatedTarget {
        LocatedTarget::new(target, self.clone())
    }
}

impl Target {
    pub fn resolve(&self, dot_default: bool) -> String {
        if self.dot.unwrap_or(dot_default) {
            format!(".{}", self.name)
        } else {
            format!("{}", self.name)
        }
    }
}
