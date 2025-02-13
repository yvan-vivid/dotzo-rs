use derive_more::derive::Constructor;
use log::debug;
use serde::de::DeserializeOwned;
use std::{
    fs::File,
    io::{BufReader, ErrorKind},
    path::{Path, PathBuf},
};
use thiserror::Error;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConfigFormat {
    Json,
    Yaml,
}

const FORMATS: &[(ConfigFormat, &str)] = &[(ConfigFormat::Yaml, "yaml"), (ConfigFormat::Json, "json")];

#[derive(Debug, Clone, Constructor, PartialEq, Eq)]
pub struct ConfigType {
    path: PathBuf,
    // TODO: Add alternates
    default_format: Option<ConfigFormat>,
}

pub struct ConfigFilePath {
    path: PathBuf,
    format: ConfigFormat,
}

pub struct ConfigFile {
    file: File,
    format: ConfigFormat,
}

#[derive(Debug, Error)]
pub enum ConfigFileReadError {
    #[error("IO error reading config file")]
    Io(#[from] std::io::Error),

    #[error("Error parsing config file")]
    JsonParsing(#[from] serde_json::Error),

    #[error("Error parsing config file")]
    YamlParsing(#[from] serde_yaml::Error),
}

pub type Result<T> = std::result::Result<T, ConfigFileReadError>;

fn try_open_file<P: AsRef<Path>>(file_path: P) -> Result<Option<File>> {
    File::open(file_path).map(Some).or_else(|e| {
        if ErrorKind::NotFound == e.kind() {
            Ok(None)
        } else {
            Err(e.into())
        }
    })
}

impl ConfigFormat {
    pub fn from_extension<S: AsRef<str>>(ext: S) -> Option<Self> {
        match ext.as_ref().to_lowercase().as_ref() {
            "json" => Some(Self::Json),
            "yaml" => Some(Self::Yaml),
            _ => None,
        }
    }
}

impl ConfigType {
    pub const fn default_yaml(path: PathBuf) -> Self {
        Self::new(path, Some(ConfigFormat::Yaml))
    }

    pub const fn default_json(path: PathBuf) -> Self {
        Self::new(path, Some(ConfigFormat::Json))
    }

    pub const fn no_default(path: PathBuf) -> Self {
        Self::new(path, None)
    }

    pub fn find_config_file<P: AsRef<Path>>(&self, path: P) -> Result<Option<ConfigFile>> {
        let path = path.as_ref();
        let base_path = path.join(&self.path);
        let mut file_paths: Vec<(ConfigFormat, PathBuf)> = FORMATS
            .iter()
            .map(|(f, ex)| (*f, base_path.with_extension(ex)))
            .collect();
        if let Some(f) = self.default_format {
            file_paths.push((f, base_path));
        }

        for (format, file_path) in file_paths {
            debug!("Looking for config file at: {}", file_path.display());
            if let Some(file) = try_open_file(&file_path)? {
                debug!("Found config file at: {}", file_path.display());
                return Ok(Some(ConfigFile { format, file }));
            }
        }
        debug!("No config file found in: {}", path.display());
        Ok(None)
    }

    pub fn override_config_file<P: AsRef<Path>>(&self, path: P) -> Option<ConfigFilePath> {
        let path = path.as_ref();
        path.extension()
            .and_then(|ext| ext.to_str())
            .and_then(ConfigFormat::from_extension)
            .map(|format| ConfigFilePath {
                format,
                path: path.into(),
            })
    }

    pub fn get_config_file<P: AsRef<Path>>(&self, path: P) -> Result<Option<ConfigFile>> {
        match self.override_config_file(path) {
            None => Ok(None),
            Some(ConfigFilePath { format, path }) => {
                try_open_file(path).map(|m| m.map(|file| ConfigFile { format, file }))
            }
        }
    }
}

impl ConfigFile {
    pub fn read_config<C: DeserializeOwned>(&self) -> Result<C> {
        let reader = BufReader::new(&self.file);
        match self.format {
            ConfigFormat::Json => serde_json::from_reader(reader).map_err(ConfigFileReadError::from),
            ConfigFormat::Yaml => serde_yaml::from_reader(reader).map_err(ConfigFileReadError::from),
        }
    }
}

pub trait ReadFromConfig: DeserializeOwned {
    fn config_type() -> ConfigType;

    fn read_from_path<P: AsRef<Path>>(path: P) -> Result<Option<Self>> {
        Self::config_type()
            .get_config_file(path)
            .and_then(|m| m.map(|p| p.read_config()).transpose())
    }

    fn find_in_path<P: AsRef<Path>>(path: P) -> Result<Option<Self>> {
        Self::config_type()
            .find_config_file(path)
            .and_then(|m| m.map(|p| p.read_config()).transpose())
    }
}
