use std::{
    io::{ErrorKind, Result},
    os::unix::fs::symlink,
    path::Path,
};

use log::{debug, info};

use super::linker::DotLink;

pub fn try_symlink<P: AsRef<Path>, Q: AsRef<Path>>(target: P, path: Q) -> Result<bool> {
    match symlink(path, target) {
        Ok(_) => Ok(true),
        Err(e) => match e.kind() {
            ErrorKind::AlreadyExists => Ok(false),
            _ => Err(e),
        },
    }
}

pub fn try_symlink_dry<P: AsRef<Path>, Q: AsRef<Path>>(target: P, path: Q) -> Result<bool> {
    let target = target.as_ref();
    let path = path.as_ref();
    Ok(if target.exists() {
        false
    } else {
        println!(
            "DRY-RUN: Would have created symlink {} => {}",
            target.display(),
            path.display()
        );
        true
    })
}

pub fn make_link(DotLink { target, link }: &DotLink) -> Result<()> {
    let link_path = &link.to_path("");

    debug!(
        "Attempting to make link {} => {}",
        target.display(),
        link_path.display()
    );
    if !try_symlink_dry(target, link_path)? {
        // TODO: Handle collision
        if target.is_symlink() {
            let current_link = target.read_link()?;
            debug!(
                "Link {} alread exists, but points to {}",
                target.display(),
                current_link.display()
            );
        } else {
            debug!("A file already exists at {}", target.display());
        }

        // TODO: Collision handling
        return Err(std::io::Error::from(std::io::ErrorKind::AlreadyExists));
    }
    info!("Linked {} => {}", target.display(), link_path.display());
    Ok(())
}

pub fn make_dir<P: AsRef<Path>>(path: P) -> Result<()> {
    let path = path.as_ref();
    info!("Creating directory: {}", path.display());
    //create_dir_all(path)
    Ok(())
}
