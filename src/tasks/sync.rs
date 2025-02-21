use log::{error, info};
use thiserror::Error;

use crate::{
    app::{cli::Cli, dotzo::App},
    components::{
        dotzo::types::Dotzo,
        environment::checks::structure::StructureCheckError,
        linker::{DotLinker, DotReconciliation},
        repo::tree::TreeTraverser,
    },
    util::prompting::{Prompter, PrompterError},
};

#[derive(Debug, Error)]
pub enum SyncTaskError {
    // TODO: Get rid of anyhow in components
    #[error("Anyhow passthrough")]
    Anywho(#[from] anyhow::Error),

    #[error("Prompt error")]
    Prompt(#[from] PrompterError),

    #[error("Structure check failure: {0}")]
    Structure(#[from] StructureCheckError),
}

pub type Result<T> = core::result::Result<T, SyncTaskError>;

pub fn sync_task<'a, APP: App<'a>>(app: &'a APP, _cli: &Cli, dotzo: Dotzo) -> Result<()> {
    // Components
    let linker = DotLinker::new(app.metadata_checks(), app.link_reader(), app.actions());
    let traverser = TreeTraverser::new(app.directory_listing(), app.metadata_checks());
    let prompting = app.prompter();
    let checks = app.structure_check();

    // Checks
    info!("Checking the environment structure");
    checks.check(&dotzo.environment)?;
    info!("Environment structure checked");

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
        let do_create_links = prompting.confirm(format!("Create {} new links?", pending.len()), false)?;
        // .with_help_message("This will create new dotfile links in home, .config, and other specified locations.")

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
