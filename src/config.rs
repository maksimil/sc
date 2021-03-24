mod config;
mod raw_config;

use std::str::FromStr;

pub use config::{Cmd, Config, Script};

use self::raw_config::RawConfig;

impl FromStr for Config {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match toml::from_str::<RawConfig>(s) {
            Ok(raw) => Config::from_raw(raw),
            Err(e) => Err(e.to_string()),
        }
    }
}

#[cfg(test)]
mod test;
