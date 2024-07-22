use crate::value;
use std::collections::HashMap;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("Invalid value: {0}")]
    ValueError(value::Error),
    #[error("Failed to convert TOML into table: {0}")]
    TomlError(toml::ser::Error),
    #[error("Failed to convert YAML into table: {0}")]
    YamlError(i32)
}

use crate::value::Value;

#[derive(Clone, Debug, Default, PartialEq)]
pub struct Table {
    pub items: HashMap<String, Value>,
}

impl Table {
    pub fn with_capacity(cap: usize) -> Self {
        Self {
            items: HashMap::with_capacity(cap),
        }
    }
    pub fn new(items: impl Into<HashMap<String, Value>>) -> Self {
        Self {
            items: items.into(),
        }
    }
    pub fn from(content: impl Into<Value>) -> Option<Self> {
        match Value::from(content.into()) {
            Value::Table(t) => Some(t),
            _ => None,
        }
    }
    pub fn from_json(content: serde_json::Value) -> Option<Self> {
        match Value::from(content) {
            Value::Table(t) => Some(t),
            _ => None,
        }
    }
    pub fn to_json(self) -> serde_json::Value {
        Value::Table(self).into()
    }
    pub fn from_toml(content: impl Into<Value>) -> Option<Self> {
        Self::from(content)
    }

    pub fn to_toml(self) -> Result<toml::Table, Error> {
        Value::Table(self)
            .try_into()
            .map_err(|e| Error::ValueError(e))
            .and_then(|v: toml::Value| toml::Table::try_from(v).map_err(|e| Error::TomlError(e)))
    }

    pub fn from_yaml(content: yaml_rust::Yaml) -> Result<Self, Error> {
        match Value::try_from(content) {
            Ok(Value::Table(t)) => Ok(t),
            Ok(_) => Err(Error::ValueError(value::Error::InvalidValue(
                "Not a table".to_owned(),
            ))),
            Err(e) => Err(Error::ValueError(e)),
        }
    }
    pub fn to_yaml(self) -> yaml_rust::Yaml {
        Value::Table(self).into()
    }
}
