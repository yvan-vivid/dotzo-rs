use derive_more::derive::Constructor;

use crate::{
    components::environment::inference::EnvironmentInference,
    util::{
        actions::Actions,
        fs::{DirectoryListing, FsRead, LinkReader, MetadataChecks},
        prompting::Prompter,
    },
};

use super::types::App;

#[derive(Debug, Constructor)]
pub struct DotzoApp<
    'a,
    MC: MetadataChecks,
    LR: LinkReader,
    DL: DirectoryListing,
    A: Actions,
    PR: Prompter,
    EI: EnvironmentInference,
> {
    metadata_checks: &'a MC,
    link_reader: &'a LR,
    directory_listing: &'a DL,
    actions: &'a A,
    prompter: &'a PR,
    inference: &'a EI,
}

impl<
        'a,
        MC: MetadataChecks,
        LR: LinkReader,
        DL: DirectoryListing,
        A: Actions,
        PR: Prompter,
        EI: EnvironmentInference,
    > App<'a> for DotzoApp<'a, MC, LR, DL, A, PR, EI>
{
    type MC = MC;
    type LR = LR;
    type DL = DL;
    type A = A;
    type PR = PR;
    type EI = EI;

    fn metadata_checks(&self) -> &'a Self::MC {
        self.metadata_checks
    }

    fn link_reader(&self) -> &'a Self::LR {
        self.link_reader
    }

    fn directory_listing(&self) -> &'a Self::DL {
        self.directory_listing
    }

    fn actions(&self) -> &'a Self::A {
        self.actions
    }

    fn prompter(&self) -> &'a Self::PR {
        self.prompter
    }

    fn inference(&self) -> &'a Self::EI {
        self.inference
    }
}

impl<'a, FS: FsRead, A: Actions, PR: Prompter, EI: EnvironmentInference> DotzoApp<'a, FS, FS, FS, A, PR, EI> {
    pub fn new_with_fs(fs: &'a FS, actions: &'a A, prompter: &'a PR, inference: &'a EI) -> Self {
        Self::new(fs, fs, fs, actions, prompter, inference)
    }
}
