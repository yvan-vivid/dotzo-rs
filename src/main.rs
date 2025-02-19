mod action;
mod app;
mod components;
mod config;
mod mapping;
mod tasks;
mod util;
mod validation;

use anyhow::Result;
use components::{dotzo::types::DotzoApp, environment::inference::DirsEnvironmentInference};

use app::{cli::parse_cli, logging::setup_logging};
use util::{
    actions::{DryFsActions, StandardFsActions},
    fs::StandardFsRead,
    prompting::InquirePrompter,
};

fn main() -> Result<()> {
    let cli = parse_cli();
    setup_logging(cli.verbose.log_level_filter())?;

    // Injectable
    let standard_fs_read = StandardFsRead::new();
    let prompter = InquirePrompter::new();
    let env_inference = DirsEnvironmentInference::new();

    if cli.dry_run {
        let standard_actions = DryFsActions::new(&standard_fs_read);
        let init = DotzoApp::new(
            &standard_fs_read,
            &standard_fs_read,
            &standard_fs_read,
            &standard_actions,
            &prompter,
            &env_inference,
        );

        let dotzo = init.init(&cli)?;
        init.run(&cli, dotzo)?;
        Ok(())
    } else {
        let standard_actions = StandardFsActions::new();
        let init = DotzoApp::new(
            &standard_fs_read,
            &standard_fs_read,
            &standard_fs_read,
            &standard_actions,
            &prompter,
            &env_inference,
        );

        let dotzo = init.init(&cli)?;
        init.run(&cli, dotzo)?;
        Ok(())
    }
}
