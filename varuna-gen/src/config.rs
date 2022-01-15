use anyhow::{Context, Result};
use serde::{
    de::{self, MapAccess, Visitor},
    Deserialize, Deserializer, Serialize,
};
use std::{collections::HashMap, env::current_dir, fmt, fmt::Debug};

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct Package {
    pub name: String,
    pub version: String,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub authors: Vec<String>,
    #[serde(default = "default_source")]
    pub source: String,
    #[serde(default = "default_template")]
    pub template: String,
    #[serde(default = "default_target")]
    pub target: String,
    #[serde(default = "default_target_prefix")]
    pub target_prefix: String,
}

fn default_source() -> String {
    "source".to_string()
}

fn default_template() -> String {
    "template".to_string()
}

fn default_target() -> String {
    "target".to_string()
}

fn default_target_prefix() -> String {
    "target".to_string()
}

#[derive(Serialize, Debug, Clone, PartialEq, Eq, Default)]
pub struct Argument {
    pub description: String,
    pub default: Option<String>,
}

#[derive(Deserialize, Debug, Clone, PartialEq, Eq, Default)]
pub struct DeArgument {
    pub description: String,
    pub default: Option<String>,
}

struct ArgumentVisitor;

impl<'de> Visitor<'de> for ArgumentVisitor {
    type Value = Argument;
    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("unexpected argument")
    }
    fn visit_str<E>(self, value: &str) -> Result<Argument, E>
    where
        E: de::Error,
    {
        Ok(Argument {
            description: value.to_string(),
            default: None,
        })
    }
    fn visit_map<A>(self, map: A) -> Result<Argument, A::Error>
    where
        A: MapAccess<'de>,
    {
        let de_argument: DeArgument =
            Deserialize::deserialize(de::value::MapAccessDeserializer::new(map))?;
        Ok(Argument {
            description: de_argument.description,
            default: de_argument.default,
        })
    }
}

impl<'de> Deserialize<'de> for Argument {
    fn deserialize<D>(deserializer: D) -> Result<Argument, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_any(ArgumentVisitor)
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct Config {
    pub package: Package,
    #[serde(default)]
    pub args: HashMap<String, Argument>,
}

impl Config {
    pub fn from_file(path: &str) -> Result<Self> {
        let file = std::fs::read_to_string(path).with_context(|| {
            format!(
                "failed to read file: {}, pwd: {:?}",
                path,
                current_dir().unwrap_or_default()
            )
        })?;
        let config: Config = toml::from_str(&file)
            .with_context(|| format!("failed to parse config file: {}", path))?;
        Ok(config)
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use super::*;

    #[test]
    fn test_deserialize_config() {
        let config = Config::from_file("example-package/example.toml").unwrap();
        let mut args = HashMap::new();
        args.insert(
            "id".to_string(),
            Argument {
                description: "student id".to_string(),
                default: None,
            },
        );
        args.insert(
            "serial".to_string(),
            Argument {
                description: "serial number".to_string(),
                default: Some("1919810".to_string()),
            },
        );
        assert_eq!(
            config,
            Config {
                package: Package {
                    name: "example-lab".to_string(),
                    version: "1.14.514".to_string(),
                    description: "This is an example lab".to_string(),
                    authors: vec![],
                    source: "source".to_string(),
                    template: "template".to_string(),
                    target: "target".to_string(),
                    target_prefix: "target".to_string(),
                },
                args,
            }
        );
    }
}
