use std::{
    fs,
    io::{self, ErrorKind},
    process::exit,
};

use clap::clap_app;
use config::Config;
use runner::run_script;

mod config;
mod runner;

const DEFAULT_SCRIPT_CONTENTS: &str = include_str!("sc_default.toml");

fn main() {
    // cli
    let matches = clap_app!(sc =>
        (version: env!("CARGO_PKG_VERSION"))
        (author: env!("CARGO_PKG_AUTHORS"))
        (about: "Manages script for your projects")
        (@arg COMMAND: "Specifies which command you want to run")
    )
    .get_matches();

    if let Some(name) = matches.value_of("COMMAND") {
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
                                eprintln!(
                                    "\u{001b}[31merror\u{001b}[0m while creating sc.toml: {}",
                                    e
                                );
                                exit(-1);
                            } else {
                                String::from(DEFAULT_SCRIPT_CONTENTS)
                            }
                        } else {
                            exit(-1);
                        }
                    } else {
                        eprintln!(
                            "\u{001b}[31merror\u{001b}[0m while parsing config file: {}",
                            e
                        );
                        exit(-1);
                    }
                }
            }
            .parse::<Config>();

            match config {
                Ok(config) => config,
                Err(e) => {
                    eprintln!("\u{001b}[31merror\u{001b}[0m: {}", e);
                    exit(-1);
                }
            }
        };

        // main functionality
        // find script
        let script = {
            match config.scripts.get(name) {
                Some(script) => script,
                None => {
                    eprintln!(
                        "\u{001b}[31merror\u{001b}[0m: script {} was not found",
                        name
                    );
                    exit(-1);
                }
            }
        };

        // run it
        eprintln!("\u{001b}[32mRunning script\u{001b}[0m {}", name);
        run_script(script);
        exit(0);
    }
}
