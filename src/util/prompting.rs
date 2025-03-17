use derive_more::derive::Constructor;
use inquire::Confirm;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum PrompterError {
    #[error("IO error")]
    General(Box<dyn core::error::Error + Send + Sync>),
}

pub type Result<T> = core::result::Result<T, PrompterError>;

pub trait Prompter {
    fn confirm(&self, message: impl AsRef<str>, default: bool) -> Result<bool>;
}

#[derive(Debug, Constructor)]
pub struct InquirePrompter {}

impl Prompter for InquirePrompter {
    fn confirm(&self, message: impl AsRef<str>, default: bool) -> Result<bool> {
        Confirm::new(message.as_ref())
            .with_default(default)
            .prompt()
            .map_err(|e| PrompterError::General(Box::new(e)))
    }
}
