use crate::dotzo::Dotzo;
use anyhow::Result;

pub fn info_task(dotzo: Dotzo) -> Result<()> {
    // TODO: Better display
    println!("Dotzo info:\n{:#?}", dotzo);
    Ok(())
}
