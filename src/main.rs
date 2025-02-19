mod app;
mod components;
mod config;
mod mapping;
mod tasks;
mod util;

use anyhow::Result;
use components::{
    dotzo::types::Dotzo,
    environment::inference::{DirsEnvironmentInference, EnvironmentInference},
    repo::types::Repo,
    validation::directory::{DirectoryCheck, DirectoryCheckError},
};

use app::{
    cli::{parse_cli, Command},
    logging::setup_logging,
};
use log::{error, info};
use tasks::{info::info_task, sync::sync_task};
use util::{actions::DryFsActions, fs::StandardFsRead};

fn main() -> Result<()> {
    let cli = parse_cli();
    setup_logging(cli.verbose.log_level_filter())?;

    // Injectable
    let standard_fs_read = StandardFsRead::new();
    let standard_actions = DryFsActions::new(&standard_fs_read);
    let env_inference = DirsEnvironmentInference::new();
    let directory_check = DirectoryCheck::new(&standard_fs_read);

    // Getting home
    info!("Identifying home directory");
    let home = env_inference.create_home(cli.home_dir)?;

    info!("Validating home directory");
    if let Err(e) = directory_check.check(&home) {
        error!("{} for home {}", e, home.as_ref().display());
        return Ok(());
    }

    let rc = env_inference.load_rc(&home)?;
    let environment = env_inference.create(home, &rc, cli.config_dir)?;
    let repo = Repo::from_config(&environment, &rc, cli.config);

    match cli.command {
        Command::Init => Ok(()),
        Command::Setup => unimplemented!(),
        Command::Sync => sync_task(Dotzo::new(environment, repo), &standard_fs_read, &standard_actions),
        Command::Info => info_task(environment, &standard_fs_read),
    }
}
