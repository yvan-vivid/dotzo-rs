use derive_more::derive::Constructor;

use crate::components::{environment::types::Environment, repo::types::Repo};

#[derive(Debug, Constructor)]
pub struct Dotzo {
    pub environment: Environment,
    pub repo: Repo,
}
