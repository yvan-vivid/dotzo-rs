use std::path::PathBuf;

use derive_more::derive::Constructor;
use relative_path::RelativePathBuf;
use serde::{Deserialize, Serialize};

use crate::config::file::{ConfigType, ReadFromConfig};

#[derive(Debug, Constructor, PartialEq, Eq, Clone, Serialize, Deserialize)]
pub struct Remote(
    // TODO: Upgrade to validated URI object
    String,
);

#[derive(Debug, Constructor, PartialEq, Eq, Clone, Serialize, Deserialize)]
pub struct Repo {
    pub location: RelativePathBuf,
    pub remote: Option<Remote>,
}

#[derive(Debug, Constructor, PartialEq, Eq, Clone, Serialize, Deserialize)]
pub struct Rc {
    pub repo: Repo,
}

impl Default for Rc {
    fn default() -> Self {
        Self {
            repo: Repo {
                location: RelativePathBuf::from("_"),
                remote: None,
            },
        }
    }
}

impl ReadFromConfig for Rc {
    fn config_type() -> ConfigType {
        ConfigType::default_yaml(PathBuf::from(".dotrc"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use indoc::indoc;

    #[test]
    fn test_deserialize_repo_remote_uri() {
        let doc = indoc! {r#"
            {
                "location": "_",
                "remote": "http://github.com/my/remote"
            }
        "#};
        let expected = Repo::new(
            RelativePathBuf::from("_"),
            Some(Remote::new("http://github.com/my/remote".into())),
        );
        assert_eq!(expected, serde_json::from_str(doc).unwrap());
    }
}
