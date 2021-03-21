use std::{fs, process::exit};

use config::{Config, Error as ConfigError};

mod config;

fn main() {
    // parse sc.toml
    let config = {
        let contents = fs::read_to_string("sc.toml");

        let config = match contents {
            Ok(contents) => contents,
            Err(e) => {
                eprintln!("Failed to parse config file with error: {}", e);
                exit(-1);
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
