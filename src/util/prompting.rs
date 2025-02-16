use std::error::Error;

use derive_more::derive::Constructor;
use inquire::{Confirm, InquireError};

pub trait Prompter {
    type Error: Error;

    fn confirm<S: AsRef<str>>(&self, message: S, default: bool) -> Result<bool, Self::Error>;
}

#[derive(Debug, Constructor)]
pub struct InquirePrompter {}

impl Prompter for InquirePrompter {
    type Error = InquireError;

    fn confirm<S: AsRef<str>>(&self, message: S, default: bool) -> Result<bool, Self::Error> {
        Confirm::new(message.as_ref()).with_default(default).prompt()
    }
}
