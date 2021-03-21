use std::{fmt, str::FromStr};

use serde::de::{Deserializer, SeqAccess, Visitor};
use serde::Deserialize;

mod script_errors {
    use super::*;

    const INVALID_NAME_CHARACTERS: [char; 2] = [' ', '\t'];

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
            test: |s| s.name.starts_with('-'),
            message: "script name cannot start with -",
        },
    ];
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum Cmd {
    Cmd(String),
    Consecutive(Vec<Cmd>),
}

struct CmdVisitor;

impl<'de> Visitor<'de> for CmdVisitor {
    type Value = Cmd;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("a string or an array of strings")
    }

    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(Cmd::Cmd(String::from(v)))
    }

    fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
    where
        A: SeqAccess<'de>,
    {
        let mut v = vec![];
        loop {
            match seq.next_element::<Cmd>() {
                Ok(Some(cmd)) => v.push(cmd),
                Ok(None) => break,
                Err(e) => return Err(e),
            }
        }

        Ok(Cmd::Consecutive(v))
    }
}

impl<'de> Deserialize<'de> for Cmd {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_any(CmdVisitor)
    }
}

#[derive(Deserialize, Debug, PartialEq, Eq, Clone)]
pub struct Script {
    // should be not "",  should not contain ' ', should not start with "-"
    pub name: String,
    pub cmd: Cmd,
    pub cwd: Option<String>,
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
                cmd: Cmd::Cmd(String::from("echo run")),
                cwd: None,
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
                    cmd: Cmd::Cmd(String::from("echo run")),
                    cwd: None,
                },
                Script {
                    name: String::from("backrun"),
                    cmd: Cmd::Cmd(String::from("echo backrun")),
                    cwd: None,
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
        name = \"-backrun\"
        cmd = \"\"

        [[scripts]]
        name = \"\"
        cmd = \"bb\"
        ",
        Err(Error::ConfigErrors(vec![
            "error in script 0:ru n : script name cannot contain tabs or spaces",
            "error in script 1:-backrun : script name cannot start with -",
            "error in script 2: : script name cannot be empty"
        ].into_iter().map(String::from).collect::<Vec<_>>()))
    }

    setup_test! {
        sequenced_commands,
        "
        [[scripts]]
        name = \"s\"
        cmd = [\"echo a\", \"echo b\"]
        
        [[scripts]]
        name = \"alpha\"
        cmd = \"echo beta\"
        ",
        Ok(Config {
            scripts: vec![
                Script {
                    name: String::from("s"),
                    cmd: Cmd::Consecutive(vec![Cmd::Cmd(String::from("echo a")),Cmd::Cmd(String::from("echo b"))]),
                    cwd: None,
                },
                Script {
                    name: String::from("alpha"),
                    cmd: Cmd::Cmd(String::from("echo beta")),
                    cwd: None,
                },
            ],
        })
    }

    setup_test! {
        set_cwd,
        "
        [[scripts]]
        name = \"alpha\"
        cmd = \"echo beta\"

        [[scripts]]
        name = \"hey\"
        cwd = \"./target\"
        cmd = [\"echo hey\", \"touch d.txt\"]
        ",
        Ok(Config {
            scripts: vec![
                Script {
                    name: String::from("alpha"),
                    cmd: Cmd::Cmd(String::from("echo beta")),
                    cwd: None,
                },
                Script {
                    name: String::from("hey"),
                    cmd: Cmd::Consecutive(vec![Cmd::Cmd(String::from("echo hey")), Cmd::Cmd(String::from("touch d.txt"))]),
                    cwd: Some(String::from("./target")),
                },
            ]
        })
    }
}
