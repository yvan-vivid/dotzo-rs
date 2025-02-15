use crate::components::environment::types::Environment;
use anyhow::Result;

pub fn info_task(environment: Environment) -> Result<()> {
    // TODO: Better display
    println!("Environment info:\n{:#?}", environment);
    Ok(())
}
