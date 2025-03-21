mod action;
mod app;
mod components;
mod config;
mod mapping;
mod tasks;
mod util;
mod validation;

use anyhow::Result;
use components::environment::inference::DirsEnvironmentInference;

use app::{cli::parse_cli, dotzo::DotzoApp, logging::setup_logging};
use tasks::run::run;
use util::{
    actions::{DryActions, StandardActions},
    fs::StandardFsRead,
    prompting::InquirePrompter,
};

fn main() -> Result<()> {
    let cli = parse_cli();
    setup_logging(cli.verbose.log_level_filter())?;

    // Injectable
    let fs_read = StandardFsRead::new();
    let prompter = InquirePrompter::new();
    let env_inference = DirsEnvironmentInference::new();

    if cli.dry_run {
        let actions = DryActions::new(&fs_read);
        let app = DotzoApp::new_with_fs(&fs_read, &actions, &prompter, &env_inference);
        run(&app, &cli)?;
    } else {
        let actions = StandardActions::new();
        let app = DotzoApp::new_with_fs(&fs_read, &actions, &prompter, &env_inference);
        run(&app, &cli)?;
    }
    Ok(())
}
