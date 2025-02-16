use derive_more::derive::Constructor;
use log::debug;
use std::path::PathBuf;
use thiserror::Error;

use crate::config::{
    file::{ConfigFileReadError, ReadFromConfig},
    rc::types::Rc,
};

use super::types::{Configs, Environment, Home};

#[derive(Debug, Error)]
pub enum EnvironmentInferenceError {
    #[error("Can't infer home directory")]
    CannotInferHome,

    #[error("Can't find rc file")]
    RcNotFound,

    #[error("Error reading rc file")]
    RcFileReadError(#[from] ConfigFileReadError),
}

pub type Result<T> = core::result::Result<T, EnvironmentInferenceError>;

pub trait EnvironmentInference {
    fn home(&self) -> Option<PathBuf>;
    fn config(&self) -> Option<PathBuf>;

    fn create_home(&self, given: Option<PathBuf>) -> Result<Home> {
        given
            .or_else(|| self.home())
            .ok_or(EnvironmentInferenceError::CannotInferHome)
            .map(Home::new)
    }

    fn create_config(&self, home: &Home, given: Option<PathBuf>) -> Configs {
        Configs::new(
            given
                .or_else(|| self.config())
                .unwrap_or_else(|| home.as_ref().join(".config")),
        )
    }

    fn load_rc(&self, home: &Home) -> Result<Rc> {
        debug!("Looking for a config in home: {}", home.as_ref().display());
        Rc::find_in_path(home)?.ok_or(EnvironmentInferenceError::RcNotFound)
    }

    fn create(&self, home: Home, _rc: &Rc, given_config: Option<PathBuf>) -> Result<Environment> {
        let config = self.create_config(&home, given_config);
        Ok(Environment::new(home, config))
    }
}

#[derive(Debug, Constructor)]
pub struct DirsEnvironmentInference {}

impl EnvironmentInference for DirsEnvironmentInference {
    fn home(&self) -> Option<PathBuf> {
        dirs::home_dir()
    }

    fn config(&self) -> Option<PathBuf> {
        dirs::config_local_dir()
    }
}

#[cfg(test)]
pub mod testing {
    use derive_more::derive::Constructor;

    use super::*;

    #[derive(Debug, Constructor)]
    pub struct TestEnvironmentInterface {
        pub test_home: Option<PathBuf>,
        pub test_config: Option<PathBuf>,
    }

    impl EnvironmentInference for TestEnvironmentInterface {
        fn home(&self) -> Option<PathBuf> {
            self.test_home.to_owned()
        }

        fn config(&self) -> Option<PathBuf> {
            self.test_config.to_owned()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::testing::*;
    use super::*;
    use std::path::{Path, PathBuf};
    use std::sync::LazyLock;

    static TEST_INFERENCE: LazyLock<TestEnvironmentInterface> = LazyLock::new(|| {
        TestEnvironmentInterface::new(Some(PathBuf::from("/test/home")), Some(PathBuf::from("/test/config")))
    });

    static TEST_INFERENCE_NO_HOME: LazyLock<TestEnvironmentInterface> =
        LazyLock::new(|| TestEnvironmentInterface::new(None, Some(PathBuf::from("/test/config"))));

    static TEST_INFERENCE_NO_CONFIG: LazyLock<TestEnvironmentInterface> =
        LazyLock::new(|| TestEnvironmentInterface::new(Some(PathBuf::from("/test/home")), None));

    static TEST_RC: LazyLock<Rc> = LazyLock::new(Rc::default);

    #[test]
    fn test_create_home_with_given_path() {
        let given_home = Some(PathBuf::from("/custom/home"));
        let result = TEST_INFERENCE.create_home(given_home);
        assert!(result.is_ok());
        assert_eq!(result.unwrap().as_ref(), Path::new("/custom/home"));
    }

    #[test]
    fn test_create_home_with_inferred_path() {
        let result = TEST_INFERENCE.create_home(None);
        assert!(result.is_ok());
        assert_eq!(result.unwrap().as_ref(), Path::new("/test/home"));
    }

    #[test]
    fn test_create_home_fails_when_no_path_available() {
        let result = TEST_INFERENCE_NO_HOME.create_home(None);
        assert!(matches!(result, Err(EnvironmentInferenceError::CannotInferHome)));
    }

    #[test]
    fn test_create_config_with_given_path() {
        let home = TEST_INFERENCE.create_home(None).unwrap();
        let given_config = Some(PathBuf::from("/custom/config"));
        let config = TEST_INFERENCE.create_config(&home, given_config);
        assert_eq!(config.as_ref(), Path::new("/custom/config"));
    }

    #[test]
    fn test_create_config_with_inferred_path() {
        let home = TEST_INFERENCE.create_home(None).unwrap();
        let config = TEST_INFERENCE.create_config(&home, None);
        assert_eq!(config.as_ref(), Path::new("/test/config"));
    }

    #[test]
    fn test_create_config_defaults_to_home_dot_config() {
        let home = TEST_INFERENCE_NO_CONFIG.create_home(None).unwrap();
        let config = TEST_INFERENCE_NO_CONFIG.create_config(&home, None);
        assert_eq!(config.as_ref(), Path::new("/test/home/.config"));
    }

    #[test]
    fn test_create_environment_with_all_paths_provided() {
        let home = Home::from(PathBuf::from("/custom/home"));
        let given_config = Some(PathBuf::from("/custom/config"));
        let result = TEST_INFERENCE.create(home, &TEST_RC, given_config);
        assert!(result.is_ok());
        let env = result.unwrap();
        assert_eq!(env.home.as_ref(), Path::new("/custom/home"));
        assert_eq!(env.config.as_ref(), Path::new("/custom/config"));
    }

    #[test]
    fn test_create_environment_with_inferred_paths() {
        let home = Home::from(PathBuf::from("/custom/home"));
        let result = TEST_INFERENCE.create(home, &TEST_RC, None);
        assert!(result.is_ok());
        let env = result.unwrap();
        assert_eq!(env.home.as_ref(), Path::new("/custom/home"));
        assert_eq!(env.config.as_ref(), Path::new("/test/config"));
    }
}
