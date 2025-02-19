use crate::components::environment::types::Environment;
use anyhow::Result;

pub fn info_task(environment: Environment) -> Result<()> {
    println!("{:#?}", environment);
    Ok(())
}
