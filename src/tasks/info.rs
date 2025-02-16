use crate::{
    components::{
        environment::{checks::EnvironmentChecker, types::Environment},
        validation::{containment::ContainmentCheck, directory::DirectoryCheck},
    },
    util::{actions::Actions, fs::FsRead, prompting::Prompter},
};
use anyhow::Result;
use log::warn;

pub fn info_task<F: FsRead, A: Actions, PR: Prompter>(
    environment: Environment,
    fs: &F,
    actions: &A,
    prompter: &PR,
) -> Result<()> {
    let directory_check = DirectoryCheck::new(fs, actions, prompter);
    let containment_check = ContainmentCheck::new(fs, fs);
    let environment_checker = EnvironmentChecker::new(&directory_check, &containment_check);
    if let Err(e) = environment_checker.check(&environment) {
        warn!("{:?}", e)
    }
    Ok(())
}
