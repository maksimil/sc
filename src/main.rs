use std::{
    fs,
    io::{self, ErrorKind},
    process::exit,
};

use config::{Config, Error as ConfigError};

mod config;

const DEFAULT_SCRIPT_CONTENTS: &str = include_str!("sc_default.toml");

fn main() {
    // parse sc.toml
    let config = {
        let contents = fs::read_to_string("sc.toml");

        let config = match contents {
            Ok(contents) => contents,
            Err(e) => {
                if e.kind() == ErrorKind::NotFound {
                    eprintln!("Configuration file not found, create sc.toml for you? [Y/N]");
                    let mut buf = String::new();
                    io::stdin().read_line(&mut buf).unwrap();
                    let buf = buf.trim();
                    if buf == "Y" || buf == "y" {
                        if let Err(e) = fs::write("sc.toml", DEFAULT_SCRIPT_CONTENTS) {
                            eprintln!("While creating sc.toml an error occured: {}", e);
                            exit(-1);
                        } else {
                            String::from(DEFAULT_SCRIPT_CONTENTS)
                        }
                    } else {
                        exit(-1);
                    }
                } else {
                    eprintln!("Failed to parse config file with error: {}", e);
                    exit(-1);
                }
            }
        }
        .parse::<Config>();

        match config {
            Ok(config) => config,
            Err(e) => match e {
                ConfigError::TomlError(e) => {
                    eprintln!("While parsing sc.toml an error occured: {}", e);
                    exit(-1);
                }
                ConfigError::ConfigErrors(e) => {
                    eprintln!("While parsing sc.toml errors occured:");

                    for msg in e {
                        eprintln!("{}", msg);
                    }

                    exit(-1);
                }
            },
        }
    };

    // execute later code
    println!("Config: {:#?}", config);
}
