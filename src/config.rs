use std::str::FromStr;

use serde::Deserialize;

const INVALID_NAME_CHARACTERS: [char; 2] = [' ', '\t'];

mod script_errors {
    use super::*;

    pub struct Test {
        pub test: fn(&Script) -> bool,
        pub message: &'static str,
    }

    pub const SCRIPT_ERRORS: [Test; 3] = [
        Test {
            test: |s| s.name == "",
            message: "script name cannot be empty",
        },
        Test {
            test: |s| s.name.contains(&INVALID_NAME_CHARACTERS[..]),
            message: "script name cannot contain tabs or spaces",
        },
        Test {
            test: |s| s.cmd == "",
            message: "script cmd cannot be empty",
        },
    ];
}

#[derive(Deserialize, Debug, PartialEq, Eq, Clone)]
pub struct Script {
    // should be not "" and should not contain ' '
    pub name: String,
    // shoudl be not ""
    pub cmd: String,
}

#[derive(Deserialize, Debug, PartialEq, Eq, Clone)]
pub struct Config {
    pub scripts: Vec<Script>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Error {
    TomlError(String),
    ConfigErrors(Vec<String>),
}

impl Config {
    fn verify(self) -> Result<Self, Error> {
        let mut errors = vec![];

        // verify scripts
        for (i, script) in self.scripts.iter().enumerate() {
            for test in script_errors::SCRIPT_ERRORS.iter() {
                if (test.test)(script) {
                    errors.push(format!(
                        "error in script {}:{} : {}",
                        i, script.name, test.message
                    ))
                }
            }
        }

        if errors.len() == 0 {
            Ok(self)
        } else {
            Err(Error::ConfigErrors(errors))
        }
    }

    pub fn find_script(&self, name: &str) -> Option<&Script> {
        self.scripts.iter().find(|s| s.name == name)
    }
}

impl FromStr for Config {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match toml::from_str::<Config>(s) {
            Ok(config) => config.verify(),
            Err(err) => Err(Error::TomlError(err.to_string())),
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    macro_rules! setup_test {
        ($test_name:ident, $pstr:expr, $result:expr) => {
            #[test]
            fn $test_name() {
                let s = $pstr;

                let sconfig = s.parse::<Config>();

                let config = $result;

                assert_eq!(sconfig, config);
            }
        };
    }

    setup_test! {
        regular_parse,
        "
        [[scripts]]
        name = \"run\"
        cmd = \"echo run\"
        ",
        Ok(Config {
            scripts: vec![Script {
                name: String::from("run"),
                cmd: String::from("echo run"),
            }],
        })
    }

    setup_test! {
        multiple_entries_parse,
        "
        [[scripts]]
        name = \"run\"
        cmd = \"echo run\"

        [[scripts]]
        name = \"backrun\"
        cmd = \"echo backrun\"
        ",
        Ok(Config {
            scripts: vec![
                Script{
                    name: String::from("run"),
                    cmd: String::from("echo run")
                },
                Script {
                    name: String::from("backrun"),
                    cmd: String::from("echo backrun"),
                }
            ]
        })
    }

    setup_test! {
        simple_toml_error,
        "
        [[scripts]]
        name = \"run\"
        cmd = 
        ",
        Err(Error::TomlError(String::from("expected a value, found a newline at line 4 column 15")))
    }

    setup_test! {
        config_errors,
        "
        [[scripts]]
        name = \"ru n\"
        cmd = \"echo run\"

        [[scripts]]
        name = \"backrun\"
        cmd = \"\"

        [[scripts]]
        name = \"\"
        cmd = \"bb\"
        ",
        Err(Error::ConfigErrors(vec![
            "error in script 0:ru n : script name cannot contain tabs or spaces",
            "error in script 1:backrun : script cmd cannot be empty",
            "error in script 2: : script name cannot be empty"
        ].into_iter().map(String::from).collect::<Vec<_>>()))
    }
}
