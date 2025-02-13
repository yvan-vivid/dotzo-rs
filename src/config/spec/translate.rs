use std::{collections::HashSet, path::Path};

use log::debug;

use crate::{
    config::file::{ReadFromConfig, Result},
    mapping::{Destination, Target, TargetMap},
};

use super::types::{Mapping, Section, Shorthand, Spec};

fn map_targets<I: IntoIterator<Item = (Section<Shorthand>, Destination)>>(sections: I) -> TargetMap {
    sections
        .into_iter()
        .flat_map(|(comp, dest)| {
            comp.unwrap_or_default()
                .into_iter()
                .map(Mapping::from)
                .map(move |Mapping { source, target, dot }| {
                    let target_filled = Target::new(target.unwrap_or_else(|| source.clone()), dot);
                    (source, dest.locate(target_filled))
                })
        })
        .collect()
}

#[derive(Debug, PartialEq, Eq)]
pub struct SpecContext {
    pub targets: TargetMap,
    pub ignores: HashSet<String>,
}

impl SpecContext {
    pub fn new(Spec { home, config, ignore }: Spec) -> Self {
        Self {
            targets: map_targets([(home, Destination::Home), (config, Destination::Config)]),
            ignores: ignore.unwrap_or_default(),
        }
    }

    pub fn from_path<P: AsRef<Path>>(path: P) -> Result<Self> {
        let path = path.as_ref();

        debug!("Finding context specification in {}", path.display());
        Spec::find_in_path(path)
            .map(|m_spec| match m_spec {
                Some(spec) => {
                    debug!("Context found");
                    spec
                }
                None => {
                    debug!("No context found, using default.");
                    Default::default()
                }
            })
            .map(Self::new)
    }
}
