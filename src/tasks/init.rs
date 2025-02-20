use log::{error, info};
use thiserror::Error;

use crate::{
    app::{cli::Cli, dotzo::App},
    components::{
        dotzo::types::Dotzo,
        environment::{
            checks::EnvironmentCheckerError,
            inference::{EnvironmentInference, EnvironmentInferenceError},
        },
        repo::types::Repo,
    },
    validation::directory::DirectoryCheckError,
};

#[derive(Debug, Error)]
pub enum InitTaskError {
    #[error("Home issue")]
    Home(#[from] DirectoryCheckError),

    #[error("Environment issue")]
    Environment(#[from] EnvironmentCheckerError),

    #[error("Environment issue")]
    EnvironmentInference(#[from] EnvironmentInferenceError),
}

pub type Result<T> = core::result::Result<T, InitTaskError>;

pub fn init_task<'a, APP: App<'a>>(app: &'a APP, cli: &Cli) -> Result<Dotzo> {
    let inference = app.inference();
    let checks = app.environment_checker(false, true);

    // Getting home
    info!("Identifying home directory");
    let home = inference.create_home(cli.home_dir.clone())?;

    info!("Validating home directory");
    checks
        .directory_checker
        .check(&home)
        .inspect_err(|e| error!("{} for home {}", e, home.as_ref().display()))
        .inspect(|_| info!("Home directory valid"))?;

    info!("Loading dotzo rc file");
    let rc = app.inference().load_rc(&home)?;

    info!("Determining the home environment");
    let environment = app.inference().create(home, &rc, cli.config_dir.clone())?;

    info!("Determining the repo");
    let repo = Repo::from_config(&environment, &rc, cli.config.clone());

    info!("Checking home structure");
    checks
        .check_structure(&environment)
        .inspect_err(|e| error!("Home structure checks failed with {}. Exiting", e))
        .inspect(|_| info!("Home structure valid"))?;

    Ok(Dotzo { environment, repo })
}
