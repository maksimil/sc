use std::{fmt, str::FromStr};

use serde::{
    de::{SeqAccess, Visitor},
    Deserialize, Deserializer,
};

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct RawConfig {
    pub scripts: Vec<RawScript>,
}

impl FromStr for RawConfig {
    type Err = toml::de::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        toml::from_str::<RawConfig>(s)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct RawScript {
    pub name: String,
    pub cmd: RawCmd,
    pub cwd: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RawCmd {
    String(String),
    List(Vec<RawCmd>),
}

struct CmdVisitor;

impl<'de> Visitor<'de> for CmdVisitor {
    type Value = RawCmd;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        write!(formatter, "a list of strings or a string")
    }

    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(RawCmd::String(String::from(v)))
    }

    fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
    where
        A: SeqAccess<'de>,
    {
        let mut v = vec![];
        loop {
            match seq.next_element::<RawCmd>() {
                Ok(Some(element)) => {
                    v.push(element);
                }
                Ok(None) => break Ok(RawCmd::List(v)),
                Err(e) => break Err(e),
            }
        }
    }
}

impl<'de> Deserialize<'de> for RawCmd {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_any(CmdVisitor)
    }
}
