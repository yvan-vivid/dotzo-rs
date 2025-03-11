use std::{cell::RefCell, io::ErrorKind, path::Path};

use derive_more::derive::Constructor;

use crate::util::{
    actions::types::{Actions, Error, Result},
    fs::{
        testing::{TestFile, TestFs},
        MetadataChecks,
    },
};

#[derive(Debug, Constructor)]
pub struct TestActions {
    pub fs: RefCell<TestFs>,
}

impl Actions for TestActions {
    fn make_dir(&self, path: impl AsRef<Path>) -> Result<()> {
        self.fs.borrow_mut().add_directory(path);
        Ok(())
    }

    fn symlink(&self, target: impl AsRef<Path>, path: impl AsRef<Path>) -> Result<()> {
        let mut fs = self.fs.borrow_mut();
        let target = target.as_ref();
        let path = path.as_ref();

        if fs.exists(target) {
            Err(Error::from_io_kind(ErrorKind::AlreadyExists))
        } else {
            fs.add_file(target.to_owned(), TestFile::Symlink(path.to_owned()));
            Ok(())
        }
    }
}
