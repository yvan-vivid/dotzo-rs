use log::{error, info};
use thiserror::Error;

use crate::{
    app::{cli::Cli, dotzo::App},
    components::{
        dotzo::types::Dotzo,
        environment::{
            checks::{home::HomeCheckError, tree::LayoutCheckError},
            inference::{EnvironmentInference, EnvironmentInferenceError},
        },
        repo::types::Repo,
    },
};

#[derive(Debug, Error)]
pub enum InitTaskError {
    #[error("Home check failure: {0}")]
    Home(#[from] HomeCheckError),

    #[error("Layout check failure: {0}")]
    Layout(#[from] LayoutCheckError),

    #[error("Environment inference failure: {0}")]
    EnvironmentInference(#[from] EnvironmentInferenceError),
}

pub type Result<T> = core::result::Result<T, InitTaskError>;

pub fn init_task<'a, APP: App<'a>>(app: &'a APP, cli: &Cli) -> Result<Dotzo> {
    let inference = app.inference();
    let home_check = app.home_check();
    let checks = app.layout_check(false, true);

    // Getting home
    info!("Identifying home directory");
    let home = inference.create_home(cli.home_dir.clone())?;

    info!("Validating home directory");
    home_check.check(&home)?;

    info!("Loading dotzo rc file");
    let rc = app.inference().load_rc(&home)?;

    info!("Determining the home environment");
    let environment = app.inference().create(home, &rc, cli.config_dir.clone())?;

    info!("Determining the repo");
    let repo = Repo::from_config(&environment, &rc, cli.config.clone());

    info!("Checking home structure");
    checks.check(&environment)?;
    info!("Home structure checked");

    Ok(Dotzo { environment, repo })
}
