use crate::components::environment::types::Environment;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum InfoTaskError {}

pub type Result<T> = core::result::Result<T, InfoTaskError>;

pub fn info_task(environment: Environment) -> Result<()> {
    println!("{:#?}", environment);
    Ok(())
}
