use anyhow::Result;
use log::info;

use crate::{
    components::{
        dotzo::types::Dotzo,
        linker::{DotLinker, DotReconciliation},
        repo::tree::TreeTraverser,
    },
    util::{
        actions::Actions,
        fs::{DirectoryListing, LinkReader, MetadataChecks},
    },
};
use inquire::Confirm;

pub fn sync_task<MC: MetadataChecks, LR: LinkReader, DL: DirectoryListing, A: Actions>(
    dotzo: Dotzo,
    mc: &MC,
    lr: &LR,
    dl: &DL,
    actions: &A,
) -> Result<()> {
    // Components
    let linker = DotLinker::new(mc, lr, actions);
    let traverser = TreeTraverser::new(dl, mc);

    // Get Mappings
    info!("Getting mappings from the repository.");
    let dot_maps = traverser.traverse(dotzo.repo.etc())?;
    let link_count = dot_maps.len();
    info!("Got {} mappings", link_count);

    // Reconciliation
    info!("Doing mapping reconciliation.");
    let DotReconciliation { confirmed, pending, .. } =
        linker.reconciliation(&dotzo.environment, dot_maps.into_values())?;

    if confirmed.len() == link_count {
        info!(
            "Confirmed all {} links already correct. Everything is synced.",
            link_count,
        );
        return Ok(());
    }
    info!(
        "Confirmed {} of {} links are already correct.",
        confirmed.len(),
        link_count
    );

    if !pending.is_empty() {
        info!("Can create {} of {} new links.", pending.len(), link_count);
        let do_create_links = Confirm::new(&format!("Create {} new links?", pending.len()))
            .with_default(false)
            .with_help_message("This will create new dotfile links in home, .config, and other specified locations.")
            .prompt()?;

        if do_create_links {
            info!("Confirmed: creating links");
            for dot_link in pending {
                linker.link(&dot_link)?
            }
        } else {
            info!("Will not create links")
        }
    }

    Ok(())
}
