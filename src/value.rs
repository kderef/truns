use std::collections::HashMap;
use thiserror::Error;
use yaml_rust::yaml;

use crate::table::Table;
use serde_json as sj;

#[derive(Error, Debug)]
pub enum Error {
    #[error("Unsupported type: '{0}'")]
    UnsupportedType(&'static str),
    #[error("Invalid value: {0}")]
    InvalidValue(String),
}

#[derive(Debug, Clone, Default, PartialEq)]
pub enum Value {
    #[default]
    Null,
    Int(i64),
    UInt(u64),
    Float(f64),
    String(String),
    Bool(bool),
    Array(Vec<Value>),
    Table(Table),
}

/***********************************************/
// JSON

impl Into<serde_json::Value> for Value {
    fn into(self) -> serde_json::Value {
        use serde_json::Value as JVal;
        match self {
            Self::Null => JVal::Null,
            Self::Bool(b) => JVal::Bool(b),
            Self::Float(f) => JVal::Number(sj::Number::from_f64(f).unwrap()),
            Self::Int(i) => JVal::Number(sj::Number::from(i)),
            Self::UInt(i) => JVal::Number(sj::Number::from(i)),
            Self::Array(a) => JVal::Array({
                let mut new_array = vec![];
                for val in a {
                    new_array.push(val.into());
                }
                new_array
            }),
            Self::String(s) => JVal::String(s),
            Self::Table(t) => JVal::Object({
                let mut items = serde_json::Map::with_capacity(t.items.len());
                for (name, val) in t.items {
                    items.insert(name, val.into());
                }

                items
            }),
        }
    }
}

impl From<serde_json::Value> for Value {
    fn from(value: sj::Value) -> Self {
        use serde_json::Value as JVal;
        match value {
            JVal::Bool(b) => Self::Bool(b),
            JVal::Null => Self::Null,
            JVal::Number(n) => {
                if n.is_f64() {
                    Self::Float(n.as_f64().unwrap())
                } else if n.is_u64() {
                    Self::UInt(n.as_u64().unwrap())
                } else {
                    Self::Int(n.as_i64().unwrap())
                }
            }
            JVal::String(s) => Self::String(s),
            JVal::Array(a) => Self::Array(a.into_iter().map(|v| v.into()).collect()),
            JVal::Object(o) => Self::Table(Table::new({
                let mut items = HashMap::with_capacity(o.len());
                for (name, val) in o {
                    items.insert(name, Self::from(val));
                }
                items
            })),
        }
    }
}

/***********************************************/
// Toml

impl From<toml::Table> for Value {
    fn from(value: toml::Table) -> Self {
        Self::from(toml::Value::Table(value))
    }
}

impl From<toml::Value> for Value {
    fn from(value: toml::Value) -> Self {
        use toml::Value as TVal;
        match value {
            TVal::Boolean(b) => Self::Bool(b),
            TVal::Datetime(dt) => Self::String(dt.to_string()),
            TVal::Float(f) => Self::Float(f),
            TVal::Integer(i) => {
                if i >= 0 {
                    Self::UInt(i as u64)
                } else {
                    Self::Int(i)
                }
            }
            TVal::String(s) => Self::String(s),
            TVal::Table(t) => Self::Table({
                let mut table = Table::with_capacity(t.len());
                for (name, val) in t {
                    table.items.insert(name, val.into());
                }
                table
            }),
            TVal::Array(a) => Self::Array(a.into_iter().map(|v| v.into()).collect()),
        }
    }
}

impl TryInto<toml::Value> for Value {
    type Error = Error;
    fn try_into(self) -> Result<toml::Value, Self::Error> {
        use toml::Value as TVal;
        Ok(match self {
            Self::Array(a) => TVal::Array(
                a.into_iter()
                    .map(TryInto::<TVal>::try_into)
                    .collect::<Result<Vec<TVal>, Self::Error>>()?,
            ),
            Self::Bool(b) => TVal::Boolean(b),
            Self::Float(f) => TVal::Float(f),
            Self::Int(i) => TVal::Integer(i),
            Self::UInt(i) => TVal::Integer(i as i64),
            Self::String(s) => TVal::String(s),
            Self::Table(t) => TVal::Table({
                let mut table = toml::Table::with_capacity(t.items.capacity());
                for (name, val) in t.items {
                    table.insert(name, val.try_into()?);
                }
                table
            }),
            Self::Null => return Err(Error::UnsupportedType("null")),
        })
    }
}


/***********************************************/
// YAML

impl Into<yaml::Yaml> for Value {
    fn into(self) -> yaml::Yaml {
        use yaml::Yaml;
        use yaml_rust::yaml::Hash;
        match self {
            Self::Null => Yaml::Null,
            Self::Bool(b) => Yaml::Boolean(b),
            Self::Float(f) => Yaml::Real(f.to_string()),
            Self::Int(i) => Yaml::Integer(i),
            Self::UInt(i) => Yaml::Integer(i as i64),
            Self::String(s) => Yaml::String(s),
            Self::Array(a) => Yaml::Array(a.into_iter().map(Into::into).collect()),
            Self::Table(t) => Yaml::Hash({
                let mut hash = Hash::with_capacity(t.items.capacity());
                for (name, val) in t.items {
                    hash.insert(Yaml::String(name), val.into());
                }
                hash
            })
        }
    }
}

impl TryFrom<yaml::Yaml> for Value {
    type Error = Error;
    fn try_from(value: yaml::Yaml) -> Result<Self, Self::Error> {
        use yaml::Yaml;
        Ok(match value {
            Yaml::Alias(_) => return Err(Error::UnsupportedType("alias")),
            Yaml::BadValue => return Err(Error::InvalidValue(format!("{value:?}"))),
            Yaml::Null => Self::Null,
            Yaml::Integer(i) if i >= 0 => Self::UInt(i as u64),
            Yaml::Integer(i) => Self::Int(i),
            Yaml::Real(fs) => Self::Float(fs.parse().unwrap()),
            Yaml::String(s) => Self::String(s),
            Yaml::Boolean(b) => Self::Bool(b),
            Yaml::Array(a) => Self::Array(
                a.into_iter()
                    .map(TryInto::try_into)
                    .collect::<Result<Vec<Self>, Self::Error>>()?,
            ),
            Yaml::Hash(h) => Self::Table(
                Table {
                    items: {
                        let mut items = HashMap::with_capacity(h.capacity());
                        for (key, val) in h {
                            match key.as_str() {
                                Some(key) => {
                                    items.insert(key.to_owned(), val.try_into()?);
                                },
                                None => return Err(Error::InvalidValue(format!("{key:?}")))
                            }
                        }
                        items
                    }
            })
        })
    }
}