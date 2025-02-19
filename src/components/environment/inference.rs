use derive_more::derive::Constructor;
use log::{debug, info};
use std::path::PathBuf;
use thiserror::Error;

use crate::{
    config::{
        file::{ConfigFileReadError, ReadFromConfig},
        rc::types::Rc,
    },
    util::dir::Labeled,
};

use super::types::{CacheDir, ConfigDir, CoreDir, DataDir, Environment, Home, StateDir};

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

pub trait DirInference<D: From<PathBuf> + Labeled> {
    fn infer(&self) -> Option<PathBuf>;

    fn inference(&self, given: Option<PathBuf>) -> Option<D> {
        given
            .inspect(|p| {
                info!(
                    "Explicitly setting the {} directory with given: {}",
                    D::LABEL,
                    p.display()
                )
            })
            .or_else(|| {
                self.infer()
                    .inspect(|p| info!("Inferring the {} directory as: {}", D::LABEL, p.display()))
            })
            .map(From::from)
    }
}

pub trait CoreInference<D: CoreDir>: DirInference<D> {
    fn inference_or_default(&self, home: &Home, given: Option<PathBuf>) -> D {
        self.inference(given).unwrap_or_else(|| {
            let de = D::from_home(home);
            info!("Using the default for the {} directory: {}", D::LABEL, de.display());
            de.into()
        })
    }
}

impl<D: CoreDir, T: DirInference<D>> CoreInference<D> for T {}

pub trait EnvironmentInference:
    DirInference<Home>
    + CoreInference<ConfigDir>
    + CoreInference<DataDir>
    + CoreInference<StateDir>
    + CoreInference<CacheDir>
{
    fn load_rc(&self, home: &Home) -> Result<Rc> {
        debug!("Looking for a config in home: {}", home.as_ref().display());
        Rc::find_in_path(home)?.ok_or(EnvironmentInferenceError::RcNotFound)
    }

    fn create_home(&self, given_home: Option<PathBuf>) -> Result<Home> {
        self.inference(given_home)
            .ok_or(EnvironmentInferenceError::CannotInferHome)
    }

    fn create(&self, home: Home, _rc: &Rc, given_config: Option<PathBuf>) -> Result<Environment> {
        let config = self.inference_or_default(&home, given_config);
        let data = self.inference_or_default(&home, None);
        let state = self.inference_or_default(&home, None);
        let cache = self.inference_or_default(&home, None);
        Ok(Environment::new(home, config, data, state, cache))
    }
}

impl<
        E: DirInference<Home>
            + CoreInference<ConfigDir>
            + CoreInference<DataDir>
            + CoreInference<StateDir>
            + CoreInference<CacheDir>,
    > EnvironmentInference for E
{
}

#[derive(Debug, Constructor)]
pub struct DirsEnvironmentInference {}

impl DirInference<Home> for DirsEnvironmentInference {
    fn infer(&self) -> Option<PathBuf> {
        dirs::home_dir()
    }
}

impl DirInference<ConfigDir> for DirsEnvironmentInference {
    fn infer(&self) -> Option<PathBuf> {
        dirs::config_local_dir()
    }
}

impl DirInference<DataDir> for DirsEnvironmentInference {
    fn infer(&self) -> Option<PathBuf> {
        dirs::data_local_dir()
    }
}

impl DirInference<StateDir> for DirsEnvironmentInference {
    fn infer(&self) -> Option<PathBuf> {
        dirs::state_dir()
    }
}

impl DirInference<CacheDir> for DirsEnvironmentInference {
    fn infer(&self) -> Option<PathBuf> {
        dirs::cache_dir()
    }
}

#[cfg(test)]
pub mod testing {
    use derive_more::derive::Constructor;

    use super::*;

    #[derive(Debug, Constructor)]
    pub struct TestEnvironmentInference {
        pub test_home: Option<PathBuf>,
        pub test_config: Option<PathBuf>,
        pub test_data: Option<PathBuf>,
        pub test_state: Option<PathBuf>,
        pub test_cache: Option<PathBuf>,
    }

    impl DirInference<Home> for TestEnvironmentInference {
        fn infer(&self) -> Option<PathBuf> {
            self.test_home.clone()
        }
    }

    impl DirInference<ConfigDir> for TestEnvironmentInference {
        fn infer(&self) -> Option<PathBuf> {
            self.test_config.clone()
        }
    }

    impl DirInference<DataDir> for TestEnvironmentInference {
        fn infer(&self) -> Option<PathBuf> {
            self.test_data.clone()
        }
    }

    impl DirInference<StateDir> for TestEnvironmentInference {
        fn infer(&self) -> Option<PathBuf> {
            self.test_state.clone()
        }
    }

    impl DirInference<CacheDir> for TestEnvironmentInference {
        fn infer(&self) -> Option<PathBuf> {
            self.test_cache.clone()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::testing::*;
    use super::*;
    use std::path::{Path, PathBuf};
    use std::sync::LazyLock;

    static TEST_INFERENCE: LazyLock<TestEnvironmentInference> = LazyLock::new(|| {
        TestEnvironmentInference::new(
            Some(PathBuf::from("/test/home")),
            Some(PathBuf::from("/test/config")),
            Some(PathBuf::from("/test/local/share")),
            Some(PathBuf::from("/test/local/state")),
            Some(PathBuf::from("/test/cache")),
        )
    });

    static TEST_INFERENCE_NO_HOME: LazyLock<TestEnvironmentInference> =
        LazyLock::new(|| TestEnvironmentInference::new(None, Some(PathBuf::from("/test/config")), None, None, None));

    static TEST_INFERENCE_NO_CONFIG: LazyLock<TestEnvironmentInference> =
        LazyLock::new(|| TestEnvironmentInference::new(Some(PathBuf::from("/test/home")), None, None, None, None));

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
        let config: ConfigDir = TEST_INFERENCE.inference_or_default(&home, given_config);
        assert_eq!(config.as_ref(), Path::new("/custom/config"));
    }

    #[test]
    fn test_create_config_with_inferred_path() {
        let home = TEST_INFERENCE.create_home(None).unwrap();
        let config: ConfigDir = TEST_INFERENCE.inference_or_default(&home, None);
        assert_eq!(config.as_ref(), Path::new("/test/config"));
    }

    #[test]
    fn test_create_config_defaults_to_home_dot_config() {
        let home = TEST_INFERENCE_NO_CONFIG.create_home(None).unwrap();
        let config: ConfigDir = TEST_INFERENCE_NO_CONFIG.inference_or_default(&home, None);
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
