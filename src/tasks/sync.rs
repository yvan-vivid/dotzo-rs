use anyhow::Result;
use log::{error, info};

use crate::{
    components::{
        dotzo::DotzoChecker,
        environment::{checks::EnvironmentChecker, types::Environment},
        linker::{DotLinker, DotReconciliation},
        repo::{tree::TreeTraverser, types::Repo},
        validation::{containment::ContainmentCheck, directory::DirectoryCheck},
    },
    util::{actions::Actions, fs::FsRead},
};
use inquire::Confirm;

pub fn sync_task<F: FsRead, A: Actions>(environment: Environment, repo: Repo, fs: &F, actions: &A) -> Result<()> {
    // Components
    let linker = DotLinker::new(fs, fs, actions);
    let traverser = TreeTraverser::new(fs, fs);
    let directory_check = DirectoryCheck::new(fs);
    let containment_check = ContainmentCheck::new(fs, fs);
    let environment_checker = EnvironmentChecker::new(&directory_check, &containment_check);
    let dotzo_checker = DotzoChecker::new(environment_checker);

    // Verification
    info!("Checking the environment");
    if let Err(e) = dotzo_checker.check(&environment, &repo) {
        error!(
            "Environment can't meet requirements to run any further. Exiting... [{}]",
            e
        );
        return Ok(());
    } else {
        info!("Dotzo environment checked")
    }

    // Get Mappings
    info!("Getting mappings from the repository.");
    let dot_maps = traverser.traverse(repo.etc())?;
    let link_count = dot_maps.len();
    info!("Got {} mappings", link_count);

    // Reconciliation
    info!("Doing mapping reconciliation.");
    let DotReconciliation { confirmed, pending, .. } = linker.reconciliation(&environment, dot_maps.into_values())?;

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
