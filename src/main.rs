mod app;
mod components;
mod config;
// mod dotzo;
mod mapping;
mod tasks;
mod util;

use anyhow::Result;
use components::{
    environment::inference::{DirsEnvironmentInference, EnvironmentInference},
    repo::types::Repo,
};

use app::{
    cli::{parse_cli, Command},
    logging::setup_logging,
};
use tasks::{info::info_task, sync::sync_task};
use util::{actions::DryFsActions, fs::StandardFsRead, prompting::InquirePrompter};

fn main() -> Result<()> {
    let cli = parse_cli();
    setup_logging(cli.verbose.log_level_filter())?;

    // Injectable
    let standard_fs_read = StandardFsRead::new();
    let standard_actions = DryFsActions::new(&standard_fs_read);
    let env_inference = DirsEnvironmentInference::new();
    let prompter = InquirePrompter::new();

    // Create dotzo
    let home = env_inference.create_home(cli.home_dir)?;
    let rc = env_inference.load_rc(&home)?;
    let environment = env_inference.create(home, &rc, cli.config_dir)?;
    let repo = Repo::from_config(&environment, &rc, cli.config);

    match cli.command {
        Command::Init => Ok(()),
        Command::Setup => unimplemented!(),
        Command::Sync => sync_task(environment, repo, &standard_fs_read, &standard_actions, &prompter),
        Command::Info => info_task(environment, &standard_fs_read, &standard_actions, &prompter),
    }
}
