use crate::{
    components::{
        environment::{checks::EnvironmentChecker, types::Environment},
        validation::{containment::ContainmentCheck, directory::DirectoryCheck},
    },
    util::fs::FsRead,
};
use anyhow::Result;
use log::warn;

pub fn info_task<F: FsRead>(environment: Environment, fs: &F) -> Result<()> {
    let directory_check = DirectoryCheck::new(fs);
    let containment_check = ContainmentCheck::new(fs, fs);
    let environment_checker = EnvironmentChecker::new(&directory_check, &containment_check);
    if let Err(e) = environment_checker.check(&environment) {
        warn!("{:?}", e)
    }
    Ok(())
}
