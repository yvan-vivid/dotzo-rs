use log::error;
use thiserror::Error;

use crate::{
    action::directory_creator::DirectoryCreator,
    app::cli::{Cli, Command},
    components::environment::{checks::EnvironmentChecker, inference::EnvironmentInference},
    tasks::{
        info::{info_task, InfoTaskError},
        init::{init_task, InitTaskError},
        sync::{sync_task, SyncTaskError},
    },
    util::{
        actions::Actions,
        fs::{DirectoryListing, FsRead, LinkReader, MetadataChecks},
        prompting::Prompter,
    },
    validation::{containment::ContainmentCheck, directory::DirectoryCheck},
};

#[derive(Debug, Error)]
pub enum DotzoAppError {
    #[error("Problem with initialization")]
    Init(#[from] InitTaskError),

    #[error("Problem with the environment")]
    Sync(#[from] SyncTaskError),

    #[error("Problem with the environment")]
    Info(#[from] InfoTaskError),
}

pub type Result<T> = core::result::Result<T, DotzoAppError>;

pub trait App<'a> {
    type MC: MetadataChecks;
    type LR: LinkReader;
    type DL: DirectoryListing;
    type A: Actions;
    type PR: Prompter;
    type EI: EnvironmentInference;

    fn metadata_checks(&self) -> &'a Self::MC;
    fn link_reader(&self) -> &'a Self::LR;
    fn directory_listing(&self) -> &'a Self::DL;
    fn actions(&self) -> &'a Self::A;
    fn prompter(&self) -> &'a Self::PR;
    fn inference(&self) -> &'a Self::EI;

    fn environment_checker(
        &self,
        yes: bool,
        create_directories: bool,
    ) -> EnvironmentChecker<'a, Self::MC, Self::LR, Self::A, Self::PR> {
        let directory_checker = DirectoryCheck::new(self.metadata_checks());
        let containment = ContainmentCheck::new(self.metadata_checks(), self.link_reader());
        let directory_creator = DirectoryCreator::new(self.actions(), self.prompter());
        EnvironmentChecker::new(
            directory_checker,
            containment,
            directory_creator,
            yes,
            create_directories,
        )
    }
}

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

impl<
        'a,
        MC: MetadataChecks,
        LR: LinkReader,
        DL: DirectoryListing,
        A: Actions,
        PR: Prompter,
        EI: EnvironmentInference,
    > DotzoApp<'a, MC, LR, DL, A, PR, EI>
{
    pub fn new(
        metadata_checks: &'a MC,
        link_reader: &'a LR,
        directory_listing: &'a DL,
        actions: &'a A,
        prompter: &'a PR,
        inference: &'a EI,
    ) -> Self {
        Self {
            metadata_checks,
            link_reader,
            directory_listing,
            actions,
            prompter,
            inference,
        }
    }

    pub fn run(&self, cli: &Cli) -> Result<()> {
        let dotzo = init_task(self, cli)?;
        match cli.command {
            Command::Init => Ok(()),
            Command::Sync => Ok(sync_task(self, cli, dotzo)?),
            Command::Info => Ok(info_task(dotzo.environment)?),
        }
    }
}

impl<'a, FS: FsRead, A: Actions, PR: Prompter, EI: EnvironmentInference> DotzoApp<'a, FS, FS, FS, A, PR, EI> {
    pub fn new_with_fs(fs: &'a FS, actions: &'a A, prompter: &'a PR, inference: &'a EI) -> Self {
        Self::new(fs, fs, fs, actions, prompter, inference)
    }
}
