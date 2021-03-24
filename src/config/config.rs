use std::collections::HashMap;

use super::raw_config::{RawCmd, RawConfig, RawScript};

macro_rules! verify {
    ($name:ident) => {
        if let Err(e) = $name {
            return Err(e);
        }
        let $name = $name.unwrap();
    };

    ($name:ident, $info:expr) => {
        if let Err(e) = $name {
            return Err((e, $info));
        }
        let $name = $name.unwrap();
    };
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Config {
    pub scripts: HashMap<String, Script>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Script {
    pub cmd: Cmd,
    pub cwd: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Cmd {
    String(String),
    Consecutive(Vec<Cmd>),
}

impl Config {
    pub fn from_raw(raw: RawConfig) -> Result<Config, String> {
        let scripts = raw.scripts.into_iter().enumerate().try_fold(
            HashMap::new(),
            |mut acc, (idx, rawscript)| match Script::from_raw(rawscript) {
                Ok((k, v)) => {
                    acc.insert(k, v);
                    Ok(acc)
                }
                Err((e, name)) => Err(format!("error in script {}:{} : {}", idx, name, e)),
            },
        );

        verify!(scripts);

        Ok(Config { scripts })
    }
}

fn name_from_raw(raw: String) -> Result<String, (String, String)> {
    if raw == "" {
        Err((String::from("script name cannot be empty"), raw))
    } else if raw.starts_with('-') {
        Err((String::from("script name cannot start with -"), raw))
    } else if raw.contains(&[' ', '\t'][..]) {
        Err((
            String::from("script name cannot contain tabs or spaces"),
            raw,
        ))
    } else {
        Ok(raw)
    }
}

impl Script {
    fn from_raw(raw: RawScript) -> Result<(String, Script), (String, String)> {
        let name = name_from_raw(raw.name);
        verify!(name);

        let cmd = Cmd::from_raw(raw.cmd);
        verify!(cmd, name);

        let cwd = raw.cwd;

        Ok((name, Script { cmd, cwd }))
    }
}

impl Cmd {
    fn from_raw(raw: RawCmd) -> Result<Cmd, String> {
        match raw {
            RawCmd::String(s) => Ok(Cmd::String(s)),
            RawCmd::List(list) => list
                .into_iter()
                .try_fold(vec![], |mut acc, e| match Cmd::from_raw(e) {
                    Ok(e) => {
                        acc.push(e);
                        Ok(acc)
                    }
                    Err(e) => Err(e),
                })
                .map(|c| Cmd::Consecutive(c)),
        }
    }
}
