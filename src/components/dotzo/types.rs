use anyhow::Result;
use derive_more::derive::Constructor;
use log::{error, info};

use crate::{
    action::directory_creator::DirectoryCreator,
    app::cli::{Cli, Command},
    components::{
        environment::{checks::EnvironmentChecker, inference::EnvironmentInference, types::Environment},
        repo::types::Repo,
    },
    tasks::{info::info_task, sync::sync_task},
    util::{
        actions::Actions,
        fs::{DirectoryListing, LinkReader, MetadataChecks},
        prompting::Prompter,
    },
    validation::{containment::ContainmentCheck, directory::DirectoryCheck},
};

#[derive(Debug, Constructor)]
pub struct Dotzo {
    pub environment: Environment,
    pub repo: Repo,
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
    pub checks: EnvironmentChecker<'a, MC, LR, A, PR>,
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
            checks: EnvironmentChecker::new(
                DirectoryCheck::new(metadata_checks),
                ContainmentCheck::new(metadata_checks, link_reader),
                DirectoryCreator::new(actions, prompter),
                false,
                true,
            ),
        }
    }

    pub fn init(&self, cli: &Cli) -> Result<Dotzo> {
        // Getting home
        info!("Identifying home directory");
        let home = self.inference.create_home(cli.home_dir.clone())?;

        info!("Validating home directory");
        if let Err(e) = self.checks.directory_checker.check(&home) {
            error!("{} for home {}", e, home.as_ref().display());
            return Err(e.into());
        }

        info!("Loading dotzo rc file");
        let rc = self.inference.load_rc(&home)?;

        info!("Determining the home environment");
        let environment = self.inference.create(home, &rc, cli.config_dir.clone())?;

        info!("Determining the repo");
        // TODO: Does this need ownership?
        let repo = Repo::from_config(&environment, &rc, cli.config.clone());

        //info!("Checking home structure");
        //if let Err(e) = self.checks.check_structure(&environment) {
        //    error!("Home structure checks failed with {}. Exiting", e);
        //    return Err();
        //}

        Ok(Dotzo { environment, repo })
    }

    pub fn run(&self, cli: &Cli, dotzo: Dotzo) -> Result<()> {
        match cli.command {
            Command::Sync => {
                info!("Checking sync requirments");
                if let Err(e) = self.checks.check_tree(&dotzo.environment) {
                    error!("Home structure checks failed with {}. Exiting", e);
                    return Ok(());
                }

                sync_task(
                    dotzo,
                    self.metadata_checks,
                    self.link_reader,
                    self.directory_listing,
                    self.actions,
                )
            }
            Command::Info => info_task(dotzo.environment),
        }
    }
}
