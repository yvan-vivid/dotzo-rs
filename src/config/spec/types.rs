use std::{collections::HashSet, path::PathBuf};

use derive_more::derive::Constructor;
use serde::{Deserialize, Serialize};

use crate::config::file::{ConfigType, ReadFromConfig};

#[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Shorthand {
    Name(String),
    Mapped(Mapping),
}

#[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize)]
pub struct Mapping {
    pub source: String,
    #[serde(default)]
    pub target: Option<String>,
    #[serde(default)]
    pub dot: Option<bool>,
}

pub type Section<T> = Option<Vec<T>>;

#[derive(Debug, Default, Clone, Constructor, PartialEq, Eq, Serialize, Deserialize)]
pub struct Spec {
    pub home: Section<Shorthand>,
    pub config: Section<Shorthand>,
    pub ignore: Option<HashSet<String>>,
}

impl From<String> for Mapping {
    fn from(source: String) -> Self {
        Self {
            source,
            target: None,
            dot: None,
        }
    }
}

impl From<Shorthand> for Mapping {
    fn from(shorthand: Shorthand) -> Self {
        match shorthand {
            Shorthand::Name(name) => name.into(),
            Shorthand::Mapped(mapping) => mapping,
        }
    }
}

impl ReadFromConfig for Spec {
    fn config_type() -> ConfigType {
        ConfigType::default_yaml(PathBuf::from(".dot"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use indoc::indoc;

    #[test]
    fn test_mapping_from_shorthand_mapping() {
        let mapping = Mapping {
            source: "source".into(),
            target: Some("target".into()),
            dot: Some(true),
        };
        assert_eq!(mapping.clone(), Shorthand::Mapped(mapping).into());
    }

    #[test]
    fn test_mapping_from_shorthand_name() {
        let mapping = Mapping {
            source: "source".into(),
            target: None,
            dot: None,
        };
        assert_eq!(mapping, Shorthand::Name("source".into()).into());
    }

    #[test]
    fn test_deserialize_simple_name() {
        let doc = indoc! {r#"
            {
                "home": ["name"]
            }
        "#};
        let expected = Spec::new(Some(vec![Shorthand::Name("name".into())]), None, None);
        assert_eq!(expected, serde_json::from_str(doc).unwrap());
    }

    #[test]
    fn test_deserialize_mapping() {
        let doc = indoc! {r#"
            {
                "config": [
                    {
                        "source": "source_name",
                        "target": "target_name",
                        "dot": true
                    }
                ]
            }
        "#};
        let expected = Spec::new(
            None,
            Some(vec![Shorthand::Mapped(Mapping {
                source: "source_name".into(),
                target: Some("target_name".into()),
                dot: Some(true),
            })]),
            None,
        );
        assert_eq!(expected, serde_json::from_str(doc).unwrap());
    }

    #[test]
    fn test_deserialize_ignore_section() {
        let doc = indoc! {r#"
            {
                "ignore": [
                    "ignoreme",
                    "dontread"
                ]
            }
        "#};
        let expected = Spec::new(
            None,
            None,
            Some(["ignoreme".into(), "dontread".into()].into_iter().collect()),
        );
        assert_eq!(expected, serde_json::from_str(doc).unwrap());
    }

    #[test]
    fn test_large_example() {
        let doc = indoc! {r#"
            {
                "home": [
                    "in_home",
                    {
                        "source": "source_name",
                        "target": "target_name"
                    },
                    {
                        "source": "original_name",
                        "dot": false
                    }
                ],
                "ignore": [
                    "ignoreme",
                    "dontread"
                ]
            }
        "#};
        let expected = Spec::new(
            Some(vec![
                Shorthand::Name("in_home".into()),
                Shorthand::Mapped(Mapping {
                    source: "source_name".into(),
                    target: Some("target_name".into()),
                    dot: None,
                }),
                Shorthand::Mapped(Mapping {
                    source: "original_name".into(),
                    target: None,
                    dot: Some(false),
                }),
            ]),
            None,
            Some(["ignoreme".into(), "dontread".into()].into_iter().collect()),
        );
        assert_eq!(expected, serde_json::from_str(doc).unwrap());
    }
}
