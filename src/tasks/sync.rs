use anyhow::Result;
use log::{error, info};

use crate::{
    components::{
        actions::make_link,
        linker::{DotLinker, DotReconciliation},
        repo::tree::TreeTraverser,
    },
    dotzo::Dotzo,
    util::fs::Fs,
};
use inquire::Confirm;

pub fn sync_task<F: Fs>(dotzo: Dotzo, fs: F) -> Result<()> {
    // Init
    //     1. load .dotrc settings or create .dotrc
    //     2. load options: home, .config, .clobber, ._
    // Init Home
    //     1. make .config and .clobber
    //     2. root link
    //     3. make `.dot_env` link with root

    // Components
    let linker = DotLinker::new(&fs, &fs);
    let traverser = TreeTraverser::new(&fs, &fs);

    if let Err(e) = dotzo.check() {
        error!("Can't meet requirements to run any further. Exiting... [{}]", e);
        return Ok(());
    } else {
        info!("Checked dotzo environment")
    }

    // Get Mappings
    info!("Getting mappings from the repository.");
    let dot_maps = traverser.traverse(dotzo.repo.etc())?;
    let link_count = dot_maps.len();
    info!("Got {} mappings", link_count);

    // Reconciliation
    let DotReconciliation { confirmed, pending, .. } = linker.reconciliation(&dotzo, dot_maps.into_values())?;

    if confirmed.len() == link_count {
        info!(
            "Confirmed all {} links already correct. Everything is synced.",
            link_count,
        );
        return Ok(());
    }
    info!("Confirmed {} of {} already correct links.", confirmed.len(), link_count);

    if !pending.is_empty() {
        info!("Can create {} of {} new links.", pending.len(), link_count);
        let do_create_links = Confirm::new(&format!("Create {} new links?", pending.len()))
            .with_default(false)
            .with_help_message("This will create new dotfile links in home, .config, and other specified locations.")
            .prompt()?;

        if do_create_links {
            info!("Confirmed: creating links");
            for dot_link in pending {
                make_link(&dot_link)?
            }
        } else {
            info!("Will not create links")
        }
    }

    Ok(())
}
